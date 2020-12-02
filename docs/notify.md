# Email Notify Service

The *Email Notify Service* handles the notification push by sending email.

## Service Profile
```json
{
    "smtp_address": "<Address of the SMTP server>",
    "username": "<The username used for SMTP authorization>",
    "password": "<The password used for SMTP authorization>",
    "email_address": "<Email address of the notification sender>",
    "name": "<Name of the notification sender>",
}
```

Only the user having a service profile of the *Email Notify Service* have permission to request these API. Request without permission will always get error response with status code `403`.

## Send a email notification.
`POST /notify/queue`

### Request
```json
{
    "receiver": "<Receiver email address>",
    "subject": "<Subject of the notification email>",
    "body": "<EMail body of the notification>"
}
```
