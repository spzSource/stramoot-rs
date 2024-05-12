GET https://api.komoot.de/v006/account/email/{{email}}/
[BasicAuth]
{{email}}: {{password}}
HTTP 200
[Captures]
user_id: jsonpath "$['username']"
token: jsonpath "$['password']"


GET https://api.komoot.de/v007/users/{{user_id}}/tours/
[QueryStringParams]
limit: 100
page: 0
type: tour_recorded
start_time: 2024-05-01T22:22:52.286126Z
[BasicAuth]
{{email}}: {{token}}
HTTP 200
