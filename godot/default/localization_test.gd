extends Control

# Create translations for both English and German languages.
func _init():
	var tr_filename = load("res://test.en.ftl")
	var tr_foldername = load("res://de/german-test.ftl")
	var tr_inline = TranslationFluent.new()
	tr_inline.locale = "pt_PT"
	var err_inline = tr_inline.append_from_text("""
-term = email
HELLO =
	{ $unreadEmails ->
		[one] Tens um { -term } por ler.
	   *[other] Tens { $unreadEmails } { -term }s por ler.
	}
	.meta = Um atributo.
""".replace("\t", "    "))
	TranslationServer.add_translation(tr_filename)
	TranslationServer.add_translation(tr_foldername)
	if err_inline == OK:
		TranslationServer.add_translation(tr_inline)
	else:
		push_error("Failed to parse tr_inline: ", error_string(err_inline))


func _on_lang_text_changed(new_text: String) -> void:
	TranslationServer.set_locale(new_text)


func _on_spin_box_value_changed(value: float) -> void:
	retranslate()


func _notification(what: int) -> void:
	if what == NOTIFICATION_TRANSLATION_CHANGED:
		retranslate()


func retranslate():
	# Default version: use a wrapper function to pass arguments:
	$Label.text = atr(TranslationFluent.args("HELLO", { "unreadEmails": $SpinBox.value }))
	# The context field is used to retrieve .attributes of a message.
	$Label2.text = atr("HELLO", "meta")
