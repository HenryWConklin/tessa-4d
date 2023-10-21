extends SceneTree

const TEST_DIR = "res://tests"

var scene_tests: Array[Script] = []
var frame_timer: int = 0
var tests_failed: int = 0

func _initialize():
	print ("Running tests")
	for file in DirAccess.get_files_at(TEST_DIR):
		if file.ends_with(".gd"):
			var test_script: GDScript = load(TEST_DIR + "/" + file)
			if file.begins_with("test_"):

				if not run_unit_test(test_script):
					tests_failed += 1
			elif file.begins_with("scenetest_"):
				scene_tests.append(test_script)
			else:
				print("Skipping non-test script: ", file)
	
	
static func run_unit_test(test_script: GDScript) -> bool:
	var any_failed = false
	for method in test_script.get_script_method_list():
		if method['name'].begins_with(UnitTest.TEST_METHOD_PREFIX):
			var instance = test_script.new()
			if instance.has_method('before_each'):
				instance.before_each()
			instance.call(method['name'])
			if instance.has_method('after_each'):
				instance.after_each()
			var test_passed = not instance._test_passed
			var message = '\t' + test_script.resource_path + "::" + method['name'] 
			if test_passed:
				message += ': passed'
			else:
				message += ': FAILED'
			print(message)
			any_failed = any_failed || not test_passed
	return not any_failed

	
func _process(delta) -> bool:
	if current_scene != null and frame_timer > 0:
		frame_timer -= 1
		if frame_timer <= 0:
			assert(false, "Scene test timed out")
			_on_test_finished(false)
	if current_scene == null:
		start_next_scene_test()
	return false
		

func start_next_scene_test():
	if scene_tests.is_empty():
		finish_tests()
		return
	var next_test: SceneTest = scene_tests.pop_back().new()
	print("Scene test: ", next_test.get_path())
	frame_timer = next_test.get_frame_limit()
	current_scene = next_test
	next_test.finished.connect(_on_test_finished)
	
func finish_tests():	
	if tests_failed == 0:
		quit(0)
	else:
		quit(1)	
		
func _on_test_finished(passed: bool):
	assert(passed, "Scene test failed")
	if not passed:
		tests_failed += 1
	current_scene.queue_free()
	current_scene = null
	
