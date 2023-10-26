extends Object
class_name ScreenshotCompare

const RECORD_FLAG = "--record-screenshots"
const PIXEL_RMS_THRESHOLD = .1
const SCREENSHOT_DIR = "res://tests/screenshots"
const GROUND_TRUTH_SUFFIX = "_truth.png"
const CANDIDATE_SUFFIX = "_candidate.png"


	
static func take_screenshot(name:String, viewport: Viewport, max_root_mean_square_difference: float = PIXEL_RMS_THRESHOLD) -> bool:
	var ground_truth_path = SCREENSHOT_DIR + "/" + name + GROUND_TRUTH_SUFFIX
	var candidate_path = SCREENSHOT_DIR + "/" + name + CANDIDATE_SUFFIX
	var candidate_image: Image = viewport.get_texture().get_image()
	
	if _record_screenshots():
		candidate_image.save_png(SCREENSHOT_DIR + "/" + name + GROUND_TRUTH_SUFFIX)
		return true

	candidate_image.save_png(candidate_path)
	if not FileAccess.file_exists(ground_truth_path):
		push_error("Ground truth image for ", name, " does not exist. Run with ", RECORD_FLAG, " to record ground truth images.")
		return false
	
	var ground_truth_image = Image.load_from_file(ground_truth_path)
	var diff_stats = ground_truth_image.compute_image_metrics(candidate_image, false)
	candidate_image.save_png(candidate_path)
	var passed = diff_stats['root_mean_squared'] <= max_root_mean_square_difference
	if not passed:
		push_error("Screenshot ", name, " did not match. Run with ", RECORD_FLAG, " to commit to the changes.")
		push_warning(diff_stats)
	return passed


	

static func _record_screenshots() -> bool:
	return OS.get_cmdline_user_args().find("--record-screenshots") != -1
