@tool
extends Node


func _init() -> void:
	print("A")
	var testgen = FluentGenerator.create()
	print("B")
	testgen.generate()
	print("C")
