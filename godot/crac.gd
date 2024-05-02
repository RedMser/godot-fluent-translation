@tool
extends Node


func _init() -> void:
	var testgen = FluentGenerator.create()
	testgen.generate()
