extends RefCounted
class_name UnitTest

const TEST_METHOD_PREFIX = "test_"

var _test_passed: bool = true

func before_each():
	pass

func after_each():
	pass

func expect(cond: bool, message: String = "Expected to be true"):
	assert(cond, message)
	if !cond:
		_test_passed = false

func expect_eq(a, b, message: String = "Expected to be equal"):
	expect(a == b, message + ": " + str(a) + " != " + str(b))
