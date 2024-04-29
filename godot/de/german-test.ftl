-term = E-mail
HELLO =
    { $unreadEmails ->
        [one] Du hast eine ungelesene { -term }.
        [13] Pech...
       *[other] Du hast { $unreadEmails } ungelesene { -term }s.
    }
    .meta = Eine Attr.