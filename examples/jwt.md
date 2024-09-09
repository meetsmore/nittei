## Json Web Token

Json Web Tokens are useful if you want your end-users to interact with the nittei API from the browser. Just upload your public RSA key (only RS256 algorithm is supported) to the `nittei` and then create jwt by signing the data with your private RSA key.

This is how your server side code might look like

```js
import jwt from "jsonwebtoken";
import { nitteiClient, Permissions } from "@meetsmore/nittei-client";

const client = nitteiClient({ apiKey: "YOUR_API_KEY" });

// Upload your public rsa signing key
await client.account.setPublicSigningKey("YOUR_PUBLIC_KEY");

// A handler in your server that generates JWT for authenticated
// users
const handleJWTRequest = async(user) => {
    // Check if your user is already associated with a
    // nittei user. Create one if not.
    if(!user.schedulerUserId) {
        const userRes = await client.user.create();
        user.schedulerUserId = userRes.data!.user.id;
    }

    const token = jwt.sign({
        // The nittei user id (the subject for this token)
        nitteiUserId: user.schedulerUserId,
        exp: 5609418990073,
        iat: 19,
        // Policy (a.k.a claims)
        schedulerPolicy: {
            allow: [Permissions.All],
            reject: [Permissions.DeleteCalendar]
        }
    }, PRIV_KEY, {
        algorithm: "RS256"
    });

    const accountRes = await client.account.me();
    const { account } = accountRes.data!;

    // Return the token back to the frontend
    return {
        token,
        accountId: account.id
    };
}

```

This is what your frontend code might look like

```js
import { nitteiUserClient } from "@meetsmore/nittei-client";

const getJWT = async() => {
    // HERE GOES LOGIC FOR CALLING YOUR SERVER ENDPOINT WHICH RETURNS A JWT AND ACCOUNT ID
}

const { token, accountId } = await getJWT();

// Construct the nittei user client
const client = nitteiUserClient({
    token,
    nitteiAccount: accountId
});

// Create calendar as user
const calendarRes = await userClient.calendar.create({
    timezone: "UTC"
});
const { calendar } = calendarRes.data!;

// This action is not allowed by the policy
const { status } = await userClient.calendar.remove(calendar.id);
console.log(status === 401);
```
