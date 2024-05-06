-term = email
HELLO =
    { $unreadEmails ->
        [one] You have one unread { -term }.
       *[other] You have { $unreadEmails } unread { -term }s.
    }
    .meta = An attr.
