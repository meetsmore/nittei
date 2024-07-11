import { Account, Calendar, INettuClient, NettuClient, User } from "../../lib";
import {
  setupAccount,
  setupUserClientForAccount,
  CREATE_ACCOUNT_CODE,
} from "../helpers/fixtures";
import { readPrivateKey, readPublicKey } from "../helpers/utils";

// This test suite is testing the specifications for our use cases

describe("Integration", () => {
  let client: INettuClient | undefined;
  let accountId: string | undefined;

  let user1: User | undefined;
  let calendar1: Calendar | undefined;

  beforeAll(async () => {
    const account = await setupAccount();

    accountId = account.accountId;
    client = account.client;
  });

  it("should create a user", async () => {
    // TODO: we cannot provide an ID for the user
    // For our use case, we would like to provide our own ID
    // To be adapted
    const res = await client?.user.create();
    if (!res?.data) {
      throw new Error("User not created");
    }
    expect(res?.status).toBe(201);

    user1 = res.data.user;
  });

  it("should create a calendar", async () => {
    if (!user1) {
      throw new Error("No user");
    }
    const res = await client?.calendar.create(user1.id, {
      timezone: "Asia/Tokyo",
    });
    expect(res?.status).toBe(201);

    if (!res?.data) {
      throw new Error("Calendar not created");
    }

    calendar1 = res.data?.calendar;
  });

  it("should not show any freebusy with no events", async () => {
    if (!user1 || !calendar1) {
      throw new Error("No user or calendar");
    }
    const res = await client?.user.freebusy(user1?.id, {
      endTs: 1000 * 60 * 60 * 24 * 4,
      startTs: 10,
      calendarIds: [calendar1.id],
    });
    if (!res?.data) {
      throw new Error("Freebusy not found");
    }
    expect(res.status).toBe(200);
    expect(res.data.busy.length).toBe(0);
  });

  it("should create one event for the user", async () => {
    if (!user1 || !calendar1) {
      throw new Error("No user or calendar");
    }
    const res = await client?.events.create(user1.id, {
      calendarId: calendar1.id,
      duration: 1000 * 60 * 60,
      startTs: 0,
      busy: true,
    });
    expect(res?.status).toBe(201);
  });

  it("should show correct freebusy with a single event in calendar", async () => {
    if (!user1 || !calendar1) {
      throw new Error("No user or calendar");
    }
    const res = await client?.user.freebusy(user1.id, {
      endTs: 1000 * 60 * 60 * 24 * 4,
      startTs: 10,
      calendarIds: [calendar1.id],
    });
    if (!res?.data) {
      throw new Error("Freebusy not found");
    }
    expect(res.data.busy.length).toBe(1);
  });
});
