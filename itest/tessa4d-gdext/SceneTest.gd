extends Node
class_name SceneTest

const DEFAULT_FRAME_LIMIT = 600

signal finished(bool)

func get_frame_limit() -> int:
	return DEFAULT_FRAME_LIMIT
