extends SceneTree

const TEST_DIR = "res://tests"
const TEST_FILE_PREFIX = "test_"

var scene_tests: Array[PackedScene] = []
var scene_done = false
var frame_timer: int = 0
var tests_failed: int = 0

func _initialize():
	print ("Running tests")
	for file in DirAccess.get_files_at(TEST_DIR):
		if not file.begins_with(TEST_FILE_PREFIX):
			continue
		if file.ends_with(".gd"):
			var test_script: GDScript = load(TEST_DIR + "/" + file)
			if not run_unit_test(test_script):
				tests_failed += 1
		elif file.ends_with(".tscn"):
			scene_tests.append(load(TEST_DIR + "/" + file))
		else:
			print("Skipping non-test file: ", file)
	
func run_unit_test(test_script: GDScript) -> bool:
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

func _process(_delta) -> bool:
	if current_scene != null and frame_timer > 0:
		frame_timer -= 1
		if frame_timer <= 0:
			push_error("Scene test timed out")
			_on_test_finished(false)
	if current_scene == null or scene_done:
		scene_done = false
		start_next_scene_test()
	return false
		

func start_next_scene_test():
	if scene_tests.is_empty():
		finish_tests()
		return
	var next_test: PackedScene = scene_tests.pop_back()
	frame_timer = SceneTest.DEFAULT_FRAME_LIMIT
	print("Scene test: ", next_test.get_path())
	self.change_scene_to_packed(next_test)
	await self.node_added
	self.current_scene.connect("finished", _on_test_finished)
	frame_timer = self.current_scene.get_frame_limit()
	
func finish_tests():	
	if tests_failed == 0:
		quit(0)
	else:
		quit(1)
		
func _on_test_finished(passed: bool):
	if not passed:
		push_error("Scene test failed")
		tests_failed += 1
	print("Scene finished")
	scene_done = true
	
	
