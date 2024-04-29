extends Control


# Create translations for both English and German languages.
func _init():
	var translation1 = TranslationFluent.new()
	translation1.locale = "en"
	# Godot automatically converts spaces to tabs for multi-line strings, but tabs are invalid in
	# FTL syntax. So convert tabs to four spaces.
	var err1 = translation1.add_bundle_from_text("""
-term = email
HELLO =
	{ $unreadEmails ->
		[one] You have one unread { -term }.
	   *[other] You have { $unreadEmails } unread { -term }s.
	}
	.meta = An attr.
""".replace("\t", "    "))
	print(err1)
	var translation2 = TranslationFluent.new()
	translation2.locale = "de"
	var err2 = translation2.add_bundle_from_text("""
-term = E-mail
HELLO =
	{ $unreadEmails ->
		[one] Du hast eine ungelesene { -term }.
		[13] Pech...
	   *[other] Du hast { $unreadEmails } ungelesene { -term }s.
	}
	.meta = Eine Attr.
""".replace("\t", "    "))
	print(err2)
	TranslationServer.add_translation(translation1)
	TranslationServer.add_translation(translation2)


func _on_lang_text_changed(new_text: String) -> void:
	TranslationServer.set_locale(new_text)


func _notification(what: int) -> void:
	if what == NOTIFICATION_TRANSLATION_CHANGED:
		retranslate()


func retranslate():
	# atr and tr have a new "args" Dictionary parameter which is used to fill $variables.
	$Label.text = atr("HELLO", { "unreadEmails": $SpinBox.value })
	# The context field is used to retrieve .attributes of a message.
	$Label2.text = atr("HELLO", {}, "meta")
