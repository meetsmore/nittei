https://github.com/11Takanori/actix-web-clean-architecture-sample

## Todos

- Logging
- More admin and user controllers for different usecases
- Delete user cleanup actions
- Freebusy queries cleanup
- Use version instead of priority on Reminder ?
- Use sorted event instace https://www.lpalmieri.com/posts/2020-12-11-zero-to-production-6-domain-modelling/
    - compatible
- core or domain ??
- Shared calendars
- https://en.wikipedia.org/wiki/Interval_scheduling
- http://www.cs.toronto.edu/~lalla/373s16/notes/ISP.pdf

## Backlog

- More api tests for [calendarevent, booking]
- More tests and docs for Reminders
- smarter mongodb schema

## Need to have a data model that will support google and outlook calendars in the future

- oauth2.0 flow with redirect to our frontend customized with account logo
- How to specify google and outlook calendar ids ?
  - on calendar level you can replicate to a selected google calendar id and outlook calendar id
  - on resource level you can specify google calendar ids and outlook calendar ids