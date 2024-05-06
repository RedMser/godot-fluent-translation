@tool
extends EditorScript


func _run() -> void:
	var generator = FluentGenerator.create()
	generator.generate()
