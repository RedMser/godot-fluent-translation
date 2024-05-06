-term = message
HELLO =
    { $unreadEmails ->
        [one] Vous avez un { -term } non lu.
       *[other] Vous avez { $unreadEmails } { -term }s non lus.
    }
    .meta = Un attribut.
