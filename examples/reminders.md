## Reminders

Your server can receive reminders before calendar events in the form of webhooks.
This means that `Nittei` will not notify your users through other means like email,
phone etc. That is supposed to be done by your server (if needed) which owns the complete user resources.

```js
import { nitteiClient, Frequenzy } from "@meetsmore/nittei-client";

const client = nitteiClient({ apiKey: "YOUR_API_KEY" });

// Set webhook url
const accountRes = await client.account.setWebhook("https://test.com/some_path");
const { account } = accountRes.data!;
// A generated key used for verifying the webhook the request
const key = account.settings.webhook!.key;


const userRes = await client.user.create();
const { user } = userRes.data!;

const calendarRes = await client.calendar.create(user.id, {
    // Starts on monday
    weekStart: 0,
    // Timezone for the calendar
    timezone: "UTC"
});
const { calendar } = calendarRes.data!;

await client.events.create(user.id, {
    calendarId: calendar.id,
    startTs: 0,
    duration: 1000 * 60 * 30, // 30 minutes in millis
    recurrence: {
        freq: Frequenzy.Daily,
        interval: 1
    },
    reminders: [{
        delta: -15, // Your webhook url will be called with this CalendarEvent 15 minutes before an occurence of this event
        identifier: "your_unqiue_identifer" // Some unique identifier that you will receive along with the webhook
    }],
    metadata: {
        mykey: "myvalue"
    }
});

// Your endpoint that nittei service will call
//  req.body = {
//      reminders: {
//          event: CalendarEvent,
//          identifier: string
//      }[]
//  }
const webhookReceiverController = (req) => {
    if(req.headers["nittei-scheduler-webhook-key"] !== key) return;
    // Handle reminder by sending email to participants or whatever your app needs to do

}

```
