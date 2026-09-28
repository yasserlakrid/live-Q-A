# ESI Welcome Day Q&A App

This project is a lightweight in-person question and help-desk system for cse (club scientifique de l'esi) welcome day. It lets visitors choose a manager, select a question topic, and submit that question to the manager in real life. Managers receive the incoming questions in real time through a WebSocket dashboard and can update each question to Waiting, In Progress, or Answered.

The application is intentionally built without a database and stores all state in memory. That makes it simple to run and easy to demo during a live event.

## Features

- Visitor flow with name capture, manager selection, and guided question picking
- Manager dashboard with status selector, queue, and live updates
- Real-time WebSocket notifications for visitors and managers
- REST API for state and question updates
- Windows XP / Frutiger Aero inspired styling with responsive layout
- In-memory state only, with no database or external service

## Architecture

- Backend: Rust + Axum + Tokio + Serde + WebSockets
- Frontend: Vanilla HTML, CSS, and JavaScript
- State: in-memory HashMap storage guarded by `Arc` and `RwLock`
- Static assets: served directly from the Rust server

## Project structure

```text
project/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── models.rs
│   ├── routes.rs
│   ├── state.rs
│   ├── websocket.rs
│   └── handlers/
│       ├── managers.rs
│       ├── questions.rs
│       └── users.rs
├── static/
│   ├── user/
│   │   ├── index.html
│   │   ├── style.css
│   │   └── app.js
│   └── manager/
│       ├── index.html
│       ├── style.css
│       └── app.js
├── README.md
└── .gitignore
```

## Manager configuration

The app initializes a built-in set of demo managers in memory:

```text
- yasser — Available
- sara — Available
- amine — Busy
- lina — Offline
```

These IDs are defined in the backend state in `src/state.rs`. They can be extended or changed easily by modifying the list of `Manager` values.

## Question definitions

The predefined question list is stored in memory in `src/state.rs` as `QuestionDefinition` records. Add or remove entries there to change the visible bubble set for visitors.

## Running the app

1. Install Rust.
2. In the project root, run:

```bash
cargo run
```

3. Open the app in a browser:
   - Visitor UI: http://localhost:3000/
   - Manager UI: http://localhost:3000/manager

The server listens on port 3000.

## REST API

### Health

```http
GET /api/health
```

Returns:

```json
{ "status": "ok" }
```

### Managers

```http
GET /api/managers
```

Returns the manager list and current availability.

### Visitor

```http
POST /api/visitors
Content-Type: application/json
```

Body:

```json
{
  "name": "Ahmed",
  "manager_id": "yasser"
}
```

### Questions

```http
GET /api/questions
POST /api/questions
GET /api/questions/:id
PATCH /api/questions/:id/status
GET /api/questions/definitions
```

Example question submission:

```json
{
  "visitor_id": "visitor-id",
  "manager_id": "yasser",
  "question": "How do clubs work?"
}
```

### Manager status

```http
PATCH /api/managers/:id/status
Content-Type: application/json
```

Body:

```json
{ "status": "Busy" }
```

## WebSocket protocol

The backend exposes a WebSocket endpoint:

```http
GET /ws?role=manager&id=yasser
GET /ws?role=visitor&id=visitor-id
```

The event payload is JSON with a `type` and `data` field.

### Example event: new question

```json
{
  "type": "new_question",
  "data": {
    "id": "uuid",
    "visitor_name": "Ahmed",
    "manager_id": "yasser",
    "manager_name": "Yasser",
    "question": "How do clubs work?",
    "created_at": "20:43",
    "status": "Waiting"
  }
}
```

### Example event: question status changed

```json
{
  "type": "question_status_changed",
  "data": {
    "id": "uuid",
    "visitor_name": "Ahmed",
    "question": "How do clubs work?",
    "status": "InProgress",
    "created_at": "20:43"
  }
}
```

### Example event: manager status changed

```json
{
  "type": "manager_status_changed",
  "data": {
    "manager_id": "yasser",
    "manager_name": "Yasser",
    "status": "Busy"
  }
}
```

## Authentication model

This app intentionally uses a lightweight, demo-friendly approach for the event setting. Managers are predefined by ID and connect to the server with a manager identity. Visitors are created on the fly. This is not intended for sensitive or private information and is designed to be easy to upgrade to a more robust auth system later.

## In-memory architecture and limitations

- All data is stored in memory.
- Restarts clear state.
- No persistence, database, or file storage is used.
- This is suitable for demos and live event use with short-lived state.

If you later want to add persistence, the natural migration would be to move the same models to a database layer while keeping the WebSocket event contract and API structure.

## Design notes

The frontend styling is intentionally inspired by classic Windows XP and Frutiger Aero concepts: soft gradients, glossy glass panels, rounded shapes, clouds, and sky-themed backgrounds. The layout is responsive and remains usable on phones, tablets, and larger screens.

## Security notes

- User-provided text is trimmed and validated server-side.
- IDs are not trusted blindly.
- Manager IDs are restricted to the known predefined set.
- WebSocket messages are parsed defensively.

## Logging

The backend logs startup events and state changes using `tracing`. Messages include connection events and question updates, without storing sensitive data.

## Example manager and question files

This repo keeps the manager and question data in the backend state instead of a database, as required for the event application. See `src/state.rs` for the source of truth.
