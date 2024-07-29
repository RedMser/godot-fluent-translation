# this is a comment
## another comment
-term = email
### third kind of comment
HELLO =
    { $unreadEmails ->
        [one] You have one unread { -term }.
       *[other] You have { $unreadEmails } unread { -term }s.
    }
    .meta = An attr. { TESTER(123, "abc", name: "yeah!") }
