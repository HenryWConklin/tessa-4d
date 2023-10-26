extends UnitTest


func test_node4d_no_parent_global_transform():
	var node = Node4D.new()
	node.transform = Transform4D.new()
	node.transform.position = Vector4(1, 2, 3, 4)
	
	var got = node.global_transform
	
	expect_eq(got.position, Vector4(1, 2, 3, 4))
	node.free()
	
func test_meshinstance4d_no_parent_global_transform():
	var node = MeshInstance4D.new()
	node.transform = Transform4D.new()
	node.transform.position = Vector4(1, 2, 3, 4)
	
	var got = node.global_transform
	
	expect_eq(got.position, Vector4(1, 2, 3, 4))
	node.free()

func test_node4d_with_parents_global_transform():
	var node1 = Node4D.new()
	node1.transform = Transform4D.new()
	node1.transform.position = Vector4(1, 2, 3, 4)
	var node2 = Node4D.new()
	node2.transform = Transform4D.new()
	node2.transform.position = Vector4(2, 3, 4, 5)
	node1.add_child(node2)
	var node3 = Node4D.new()
	node3.transform = Transform4D.new()
	node3.transform.position = Vector4(3, 4, 5, 6)
	node2.add_child(node3)
	
	var got = node3.global_transform
	
	expect_eq(got.position, Vector4(6, 9, 12, 15))
	node1.free()
	
func test_meshinstance4d_with_parents_global_transform():
	var node1 = Node4D.new()
	node1.transform = Transform4D.new()
	node1.transform.position = Vector4(1, 2, 3, 4)
	var node2 = Node4D.new()
	node2.transform = Transform4D.new()
	node2.transform.position = Vector4(2, 3, 4, 5)
	node1.add_child(node2)
	var node3 = MeshInstance4D.new()
	node3.transform = Transform4D.new()
	node3.transform.position = Vector4(3, 4, 5, 6)
	node2.add_child(node3)
	
	var got = node3.global_transform
	
	expect_eq(got.position, Vector4(6, 9, 12, 15))
	node1.free()

func test_node4d_compose_order():
	var node1 = Node4D.new()
	node1.transform = Transform4D.new()
	var rotation = Bivector4D.new()
	rotation.xy = PI / 2
	node1.transform.set_rotor(Rotor4D.from_bivector_angles(rotation))
	var node2 = Node4D.new()
	node2.transform = Transform4D.new()
	node2.transform.position = Vector4(1, 2, 3, 4)
	node1.add_child(node2)

	var got = node2.global_transform

	expect(got.position.is_equal_approx(Vector4(-2, 1, 3, 4)))
	node1.free()

func test_node4d_set_global_transform():
	var node1 = Node4D.new()
	node1.transform = Transform4D.new()
	var rotation = Bivector4D.new()
	rotation.xy = PI / 2
	node1.transform.set_rotor(Rotor4D.from_bivector_angles(rotation))
	var node2 = Node4D.new()
	node1.add_child(node2)
	var target_transform =  Transform4D.new()
	target_transform.position = Vector4(1, 2, 3, 4)

	node2.global_transform = target_transform

	expect(node2.global_transform.position.is_equal_approx(target_transform.position), "Expected global transform to match")
	expect(node2.transform.position.is_equal_approx(Vector4(2, -1, 3, 4)), "Expected local transform to match")
	node1.free()
