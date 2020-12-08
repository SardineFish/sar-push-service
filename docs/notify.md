# Email Notify Service

The *Email Notify Service* handles the notification push by sending email.

## Service Profile
```json
{
    "smtp_address": "<Address of the SMTP server, e.g. smtp.example.com:25>",
    "tls": "<Whether use TLS connect to SMTP server. true | false>",
    "username": "<The username used for SMTP authorization>",
    "password": "<The password used for SMTP authorization>",
    "email_address": "<Email address of the notification sender>",
    "name": "<Display name of the notification sender>",
}
```

Only the user with an *Email Notify Service* profile can be accessible to request these API, otherwise will result in a `403` response with error message.

## Send a email notification.
`POST /notify/queue`

### Request
```json
{
    "to": "<Receiver email address>",
    "subject": "<Subject of the notification email>",
    "content_type": "<Content-Type in the mail header>",
    "body": "<EMail body of the notification>"
}
```

### Response
```json
{
    "message_id": "<An unique ID of the message>",
    "status": "<Mail status, Pending | Sent | Error>",
    "error": "[Error message if status == Error]"
}
```

----------------

## List all notification
`GET /notify/all/{uid}?filter=<status>`

List all notification of a specific user.

### Query Parameters

| Param  | Type | Description |
|--------|------|-------------|
| filter | `Enum` ( `All` \| `Pending` \| `Sent` \| `Error` ) | List only the nofications status match the filter

### Request
No request data required.

### Response
```json
[
    {
        "message_id": "<An unique ID of the message>",
        "status": "<Mail status, Pending | Sent | Error>",
        "error": "[Error message if status == Error]"
    },
    {
        "message_id": "<Another unique ID of the message>",
        "status": "<Mail status, Pending | Sent | Error>",
        "error": "[Error message if status == Error]"
    },
    "...",
]
```

### Error 
If the user specific by `uid` dose not exists or dose not have a notify service, 404 will be response.

----------------

## Query the status of a specific notification email
`GET /notify/{message_id}`

### Request
No request data required.

### Response
```json
{
    "message_id": "<An unique ID of the message>",
    "status": "<Mail status, Pending | Sent | Error>",
    "error": "[Error message if status == Error]"
}
```