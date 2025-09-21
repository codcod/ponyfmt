/*
Message domain models and small helpers.

Defines message kinds (`EmailKind`, `SmsKind`, `PushKind`) and the
`OutboundMessage` trait implemented by concrete `EmailMessage`,
`SmsMessage` and `PushMessage` value classes. These types are
immutable (`class val`) and contain only simple accessors.

Also provides `ProcessedMessage` which wraps an `OutboundMessage`
with processing metadata (a `Map[String, String]`). The `Router` and
gateways operate against `ProcessedMessage` instances.

Public API:
- `MessageKind`: union of available message kinds.
- `OutboundMessage` trait: `kind()`, `recipient()`, `body()`, `id()`.
- `EmailMessage`, `SmsMessage`, `PushMessage`: simple value classes.
- `ProcessedMessage`: wrapper with `original` and `metadata`.
*/

use "collections"

primitive EmailKind
primitive SmsKind
primitive PushKind

type MessageKind is (EmailKind | SmsKind | PushKind)

trait val OutboundMessage
  fun kind(): MessageKind
  fun recipient(): String
  fun body(): String
  fun id(): U64

class val EmailMessage is OutboundMessage
  let _id: U64
  let _to: String
  let _body: String
  let _subject: String
  new val create(id': U64, to': String, body': String, subject': String) =>
    _id = id'; _to = to'; _body = body'; _subject = subject'
  fun id(): U64 => _id
  fun kind(): MessageKind => EmailKind
  fun recipient(): String => _to
  fun body(): String => _body
  fun subject(): String => _subject

class val SmsMessage is OutboundMessage
  let _id: U64
  let _to: String
  let _body: String
  new val create(id': U64, to': String, body': String) =>
    _id = id'; _to = to'; _body = body'
  fun id(): U64 => _id
  fun kind(): MessageKind => SmsKind
  fun recipient(): String => _to
  fun body(): String => _body

class val PushMessage is OutboundMessage
  let _id: U64
  let _to: String
  let _body: String
  new val create(id': U64, to': String, body': String) =>
    _id = id'; _to = to'; _body = body'
  fun id(): U64 => _id
  fun kind(): MessageKind => PushKind
  fun recipient(): String => _to
  fun body(): String => _body

// Envelope post-processing
class val ProcessedMessage
  let original: OutboundMessage
  let metadata: Map[String, String] val
  new val create(orig: OutboundMessage, meta: Map[String, String] val) =>
    original = orig
    metadata = meta