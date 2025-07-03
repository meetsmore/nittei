# Retry Mechanism for GET Requests

The Nittei JavaScript client supports automatic retry for GET requests when encountering connection errors like timeouts or connection resets.

## Configuration

You can configure the retry mechanism when creating a client:

```typescript
import { NitteiUserClient, type RetryConfig } from "@meetsmore/nittei";

const retryConfig: RetryConfig = {
  enabled: true,
  maxRetries: 3, // Maximum number of retry attempts (default: 3)
  baseDelay: 300, // Base delay in milliseconds (default: 300)
  maxDelay: 5000, // Maximum delay in milliseconds (default: 5000)
};

const client = NitteiUserClient({
  apiKey: "your-api-key",
  retry: retryConfig,
});
```

## How It Works

The retry mechanism uses exponential backoff:

- **Attempt 1**: Immediate request
- **Attempt 2**: Wait 1 second (baseDelay)
- **Attempt 3**: Wait 2 seconds (baseDelay \* 2^1)
- **Attempt 4**: Wait 4 seconds (baseDelay \* 2^2)

The delay is capped at `maxDelay` to prevent excessive waiting times.

## Retryable Errors

The following errors will trigger a retry:

- `ECONNRESET` - Connection reset
- `ECONNABORTED` - Connection aborted
- `ETIMEDOUT` - Request timeout
- `ENOTFOUND` - DNS lookup failed
- `ENETUNREACH` - Network unreachable
- Any error message containing "timeout", "network", or "connection"

## Example Usage

```typescript
// Enable retry for all GET requests
const client = NitteiUserClient({
  apiKey: "your-api-key",
  retry: {
    enabled: true,
    maxRetries: 3,
    baseDelay: 1000,
  },
});

// This GET request will automatically retry on connection errors
try {
  const user = await client.user.me();
  console.log("User:", user);
} catch (error) {
  // If all retries fail, the original error is thrown
  console.error("Failed after retries:", error);
}
```

## Admin Client

The retry mechanism also works with the admin client:

```typescript
import { NitteiClient } from "@meetsmore/nittei";

const client = await NitteiClient({
  apiKey: "your-api-key",
  retry: {
    enabled: true,
    maxRetries: 5,
    baseDelay: 500,
    maxDelay: 5000,
  },
});

// All GET requests will retry on connection errors
const account = await client.account.me();
```

## Notes

- Only GET requests are retried (POST, PUT, DELETE requests are not retried)
- HTTP status codes (4xx, 5xx) are not retried - only connection-level errors
- The retry mechanism is enabled by default (`enabled: true`)
- Each client instance can have its own retry configuration
