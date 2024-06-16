use bevy::{
    app::{App, PluginsState},
    asset::{AssetServer, Assets, Handle, LoadState},
    ecs::{
        entity::Entity,
        query::With,
        system::{Query, Res, ResMut, Resource},
        world::World,
    },
    render::{texture::Image, view::screenshot::ScreenshotManager},
    window::PrimaryWindow,
};
use image::DynamicImage;
use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    thread::sleep,
    time::Duration,
};

const EXPECTED_FILE_SUFFIX: &'static str = "-expected.png";
const ACTUAL_FILE_SUFFIX: &'static str = "-actual.png";

#[derive(Resource, Clone)]
pub struct ScreenshotTestInfo {
    path: PathBuf,
    counter: usize,
    expected_screenshot_handles: Vec<Handle<Image>>,
    num_compared: Arc<Mutex<usize>>,
    any_failed: Arc<Mutex<bool>>,
}

/// Sets up necessary state to capture and compare screenshots in a test.
/// Use [`take_screenshot`] to capture and compare a screenshot.
pub fn setup_screenshots(world: &mut World, name: &str) {
    let path = std::path::Path::new("screenshots").join(name);
    std::fs::create_dir_all(&Path::new("assets").join(&path)).unwrap();
    let mut asset_server = world
        .get_resource_mut::<AssetServer>()
        .expect("world to have AssetServer for image loading");
    let screenshot_info = ScreenshotTestInfo {
        counter: 0,
        expected_screenshot_handles: load_expected_screenshots(&mut asset_server, &path),
        path,
        num_compared: Arc::new(Mutex::new(0)),
        any_failed: Arc::new(Mutex::new(false)),
    };
    world.insert_resource(screenshot_info);
}

/// System that takes a test screenshot and asynchronously compares it against its expected value.
/// Typically run with `app.world.run_system_once(take_screenshot)` before calling `app.update()` to advance the frame.
/// Be sure to call [`setup_screenshots`] before using this system.
pub fn take_screenshot(
    mut screenshot_test_info: ResMut<ScreenshotTestInfo>,
    main_window: Query<Entity, With<PrimaryWindow>>,
    mut screenshot_manager: ResMut<ScreenshotManager>,
    images: Res<Assets<Image>>,
) {
    bevy::log::info!("Taking screenshot {}", screenshot_test_info.counter);
    let window = main_window.get_single().unwrap();
    let screenshot_id = screenshot_test_info.counter;
    let filename = format!("{:04}{}", screenshot_id, ACTUAL_FILE_SUFFIX);
    let path = Path::new("assets")
        .join(&screenshot_test_info.path)
        .join(filename);
    let expected_image = screenshot_test_info
        .expected_screenshot_handles
        .get(screenshot_id)
        .and_then(|handle| images.get(handle))
        .cloned();
    let any_failed = screenshot_test_info.any_failed.clone();
    let num_compared = screenshot_test_info.num_compared.clone();
    screenshot_manager
        .take_screenshot(window, move |image| {
            let span = bevy::log::error_span!("compare screenshots", screenshot_id = screenshot_id);
            span.in_scope(|| {
                let mut num_compared_lock = num_compared.lock().unwrap();
                let actual = image.try_into_dynamic().unwrap();
                actual.save(path).unwrap();
                if let Some(expected_image) = expected_image {
                    let expected = expected_image.try_into_dynamic().unwrap();
                    if !images_match(actual, expected) {
                        *any_failed.lock().unwrap() = true;
                    }
                }
                *num_compared_lock += 1;
            })
        })
        .unwrap();
    screenshot_test_info.counter += 1;
}

/// Blocks until the `app` is ready for testing.
pub fn wait_ready(app: &mut App) {
    while app.plugins_state() != PluginsState::Ready {
        sleep(Duration::from_millis(100));
    }
    app.finish();
    app.cleanup();

    // Block on async loading of expected screenshots.
    if let Some(screenshot_info) = app.world.get_resource::<ScreenshotTestInfo>() {
        let screenshot_info = screenshot_info.clone();
        bevy::log::info!("Waiting for expected screenshots");
        loop {
            let asset_server = app.world.get_resource::<AssetServer>().unwrap();
            if check_loaded(&asset_server, &screenshot_info.expected_screenshot_handles) {
                break;
            }
            // Just sleeping never actually loads the images, need to let update run a few times.
            app.update();
        }
        bevy::log::info!("Loaded expected screenshots");
    }

    // Without this loop, nothing renders in the test ~50% of the time. Stall for the window to set itself up or something.
    for _ in 1..100 {
        app.update();
    }
}

const MAX_DIFF_PIXELS: usize = 100;
const MAX_PIXEL_DIFF: u8 = 3;
/// Checks if two images match withing some pre-defined similarity thresholds.
/// Logs errors if the images do not match.
pub fn images_match(actual: DynamicImage, expected: DynamicImage) -> bool {
    bevy::log::info!("Comparing");
    if actual.width() != expected.width() || actual.height() != expected.height() {
        bevy::log::error!(
            "Image dimensions do not match: {}x{} != {}x{}",
            actual.width(),
            actual.height(),
            expected.width(),
            expected.height()
        );
        return false;
    }
    let mut diff_pixels = 0;
    let actual_rgb8 = actual.into_rgb8();
    let expected_rgb8 = expected.into_rgb8();
    for (x, y, actual_color) in actual_rgb8.enumerate_pixels() {
        let expected_color = expected_rgb8.get_pixel(x, y);
        let rdiff = actual_color.0[0].abs_diff(expected_color.0[0]);
        let gdiff = actual_color.0[1].abs_diff(expected_color.0[1]);
        let bdiff = actual_color.0[2].abs_diff(expected_color.0[2]);
        if rdiff.max(gdiff).max(bdiff) > MAX_PIXEL_DIFF {
            diff_pixels += 1;
        }
    }

    if diff_pixels >= MAX_DIFF_PIXELS {
        bevy::log::error!("Too many changed pixels: {}", diff_pixels);
        return false;
    }

    bevy::log::info!(
        different_pixels = diff_pixels,
        max_different_pixels = MAX_DIFF_PIXELS,
        max_pixel_difference = MAX_PIXEL_DIFF,
        "Screenshots match"
    );
    true
}

/// Checks that all test screenshots taken in the given app match their expected values, panics if any screenshot is invalid.
/// Will run `app.update()` in order to advance async jobs doing the screenshot comparisons.
pub fn check_screenshots(app: &mut App) {
    // Block until all screenshots are taken and compared.
    loop {
        {
            let screenshot_info = app.world.get_resource::<ScreenshotTestInfo>().unwrap();
            let num_screenshots = screenshot_info.counter;
            let compared = screenshot_info.num_compared.lock().unwrap();
            bevy::log::info!("Compared {}/{}", compared, num_screenshots);
            if *compared >= num_screenshots {
                break;
            }
        }
        // Need to run update for async stuff to run.
        app.update();
    }
    let screenshot_info = app.world.get_resource::<ScreenshotTestInfo>().unwrap();
    if *screenshot_info.any_failed.lock().unwrap() {
        panic!("Screenshot comparison failed");
    }
    // Check this after letting all the comparisons run for maximum feedback.
    assert_eq!(
        screenshot_info.counter,
        screenshot_info.expected_screenshot_handles.len(),
        "unexpected number of screenshots"
    );
}

/// Loads ground truth screenshots from the assets folder. Returns handles to them in order of their IDs.
fn load_expected_screenshots(asset_server: &mut AssetServer, path: &Path) -> Vec<Handle<Image>> {
    if let Ok(paths) = std::fs::read_dir(Path::new("assets").join(path)) {
        let mut paths: Vec<_> = paths
            .into_iter()
            .flat_map(|maybe_path| maybe_path.into_iter())
            .filter(|path| {
                path.file_name()
                    .to_string_lossy()
                    .ends_with(EXPECTED_FILE_SUFFIX)
            })
            .map(|entry| entry.path())
            .collect();
        // Should probably parse out the ID, but this works with the current format.
        paths.sort_by(|a, b| a.file_name().cmp(&b.file_name()));

        return paths
            .into_iter()
            .map(|path| {
                bevy::log::info!("Loading expected screenshot {}", path.display());
                let bevy_path = path.strip_prefix("assets").unwrap().to_owned();
                asset_server.load(bevy_path)
            })
            .collect();
    }

    return vec![];
}

/// Checks if a list of [`Image`]s have finished loading.
fn check_loaded(asset_server: &AssetServer, handles: &[Handle<Image>]) -> bool {
    for handle in handles {
        bevy::log::debug!(
            "Checking loaded image: {:?}, state {:?}",
            handle,
            asset_server.get_load_state(handle.id())
        );
        match asset_server.get_load_state(handle.id()).unwrap() {
            LoadState::Loading | LoadState::NotLoaded => {
                return false;
            }
            LoadState::Failed => panic!("Failed to load expected screenshot: {:?}", handle),
            _ => {}
        }
    }
    return true;
}
