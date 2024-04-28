extends Control


func _init():
	var translation1 = TranslationFluent.new()
	translation1.locale = "en"
	var err1 = translation1.add_bundle_from_text(&"en", """
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
	var err2 = translation2.add_bundle_from_text(&"de", """
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
	$Label.text = atr("HELLO", { "unreadEmails": $SpinBox.value })
	$Label2.text = atr("HELLO", {}, "meta")
