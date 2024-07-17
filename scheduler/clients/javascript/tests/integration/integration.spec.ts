import { Calendar, INettuClient, User, CalendarEvent } from "../../lib";
import { setupAccount } from "../helpers/fixtures";

// This test suite is testing the specifications for our use cases

describe("Integration", () => {
  let client: INettuClient | undefined;
  let accountId: string | undefined;

  let user1: User | undefined;
  let user1Calendar1: Calendar | undefined;
  let user1Calendar1Event1: CalendarEvent | undefined;
  let user1Calendar2: Calendar | undefined;
  // let user1Calendar2Event1: CalendarEvent | undefined;

  beforeAll(async () => {
    const account = await setupAccount();

    accountId = account.accountId;
    client = account.client;
  });

  describe("Basic functionalities", () => {
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

      user1Calendar1 = res.data?.calendar;
    });

    it("should list the events in the calendar and get an empty array", async () => {
      if (!user1Calendar1) {
        throw new Error("No calendar");
      }
      const startTs = 10;
      const endTs = 1000 * 60 * 60 * 24 * 4;

      const res = await client?.calendar.getEvents(
        user1Calendar1.id,
        startTs,
        endTs
      );

      expect(res?.status).toBe(200);
      expect(res?.data).toBeDefined();
      expect(res?.data?.events.length).toBe(0);
    });

    it("should query freebusy and get no events", async () => {
      if (!user1 || !user1Calendar1) {
        throw new Error("No user or calendar");
      }
      const res = await client?.user.freebusy(user1?.id, {
        startTs: 10,
        endTs: 1000 * 60 * 60 * 24 * 4,
        calendarIds: [user1Calendar1.id],
      });
      if (!res?.data) {
        throw new Error("Freebusy not found");
      }
      expect(res.status).toBe(200);
      expect(res.data.busy.length).toBe(0);
    });

    it("should create one event for the user", async () => {
      if (!user1 || !user1Calendar1) {
        throw new Error("No user or calendar");
      }
      const res = await client?.events.create(user1.id, {
        calendarId: user1Calendar1.id,
        duration: 1000 * 60 * 60,
        startTs: 0,
        busy: true,
      });
      expect(res?.status).toBe(201);
      user1Calendar1Event1 = res?.data?.event;
    });

    it("should list the events in the calendar and get one event", async () => {
      if (!user1Calendar1) {
        throw new Error("No calendar");
      }
      const startTs = 10;
      const endTs = 1000 * 60 * 60 * 24 * 4;

      const res = await client?.calendar.getEvents(
        user1Calendar1.id,
        startTs,
        endTs
      );

      expect(res?.status).toBe(200);
      expect(res?.data).toBeDefined();
      expect(res?.data?.events.length).toBe(1);
      expect(res?.data?.events[0].event.id).toEqual(user1Calendar1Event1?.id);
    });

    it("should show correct freebusy with a single event in calendar", async () => {
      if (!user1 || !user1Calendar1) {
        throw new Error("No user or calendar");
      }
      const res = await client?.user.freebusy(user1.id, {
        endTs: 1000 * 60 * 60 * 24 * 4,
        startTs: 10,
        calendarIds: [user1Calendar1.id],
      });
      if (!res?.data) {
        throw new Error("Freebusy not found");
      }
      expect(res.data.busy.length).toBe(1);
    });
  });

  describe("With 2 calendars", () => {
    it("should create a 2nd calendar", async () => {
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

      user1Calendar2 = res.data?.calendar;
    });

    it("should list the events in the 2nd calendar and get an empty array", async () => {
      if (!user1Calendar2) {
        throw new Error("No calendar");
      }
      const startTs = 10;
      const endTs = 1000 * 60 * 60 * 24 * 4;

      const res = await client?.calendar.getEvents(
        user1Calendar2.id,
        startTs,
        endTs
      );

      expect(res?.status).toBe(200);
      expect(res?.data).toBeDefined();
      expect(res?.data?.events.length).toBe(0);
    });
  });

  describe("With multiple users", () => {
    let user2: User | undefined;
    let calendar2: Calendar | undefined;

    beforeAll(async () => {
      const res = await client?.user.create();
      if (!res?.data) {
        throw new Error("User not created");
      }
      user2 = res.data.user;

      const res2 = await client?.calendar.create(user2.id, {
        timezone: "Asia/Tokyo",
      });
      if (!res2?.data) {
        throw new Error("Calendar not created");
      }
      calendar2 = res2.data.calendar;

      if (!user2 || !calendar2) {
        throw new Error("No user or calendar");
      }
      const resEvent = await client?.events.create(user2.id, {
        calendarId: calendar2.id,
        duration: 1000 * 60 * 60,
        startTs: 1000 * 60 * 60,
        busy: true,
      });
      expect(resEvent?.status).toBe(201);
    });

    it.todo("should show correct freebusy for both users (admin)");
  });
});
