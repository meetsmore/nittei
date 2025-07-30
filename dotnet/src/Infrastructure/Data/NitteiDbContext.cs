using Microsoft.EntityFrameworkCore;
using Nittei.Domain;
using Nittei.Domain.Shared;
using System.Text.Json;

namespace Nittei.Infrastructure.Data;

/// <summary>
/// Entity Framework DbContext for Nittei
/// </summary>
public class NitteiDbContext : DbContext
{
    public NitteiDbContext(DbContextOptions<NitteiDbContext> options) : base(options)
    {
    }

    public DbSet<Account> Accounts { get; set; }
    public DbSet<User> Users { get; set; }
    public DbSet<UserIntegration> UserIntegrations { get; set; }
    public DbSet<Calendar> Calendars { get; set; }
    public DbSet<CalendarEvent> CalendarEvents { get; set; }
    public DbSet<Service> Services { get; set; }
    public DbSet<Schedule> Schedules { get; set; }

    public DbSet<Reminder> Reminders { get; set; }

    protected override void OnModelCreating(ModelBuilder modelBuilder)
    {
        base.OnModelCreating(modelBuilder);

        // Ignore value objects that should not be treated as entities
        modelBuilder.Ignore<TimePlan>();
        modelBuilder.Ignore<ServiceResource>();
        modelBuilder.Ignore<ServiceWithUsers>();
        modelBuilder.Ignore<BusyCalendarProvider>();
        modelBuilder.Ignore<SyncedCalendar>();
        modelBuilder.Ignore<SyncedCalendarEvent>();
        modelBuilder.Ignore<EventGroup>();

        // Configure Account entity
        modelBuilder.Entity<Account>(entity =>
        {
            entity.ToTable("accounts");
            entity.HasKey(e => e.Id);
            entity.Property(e => e.Id).HasColumnName("account_uid").HasConversion(
                id => (Guid)id,
                guid => (Id)guid);
            entity.Property(e => e.SecretApiKey).HasColumnName("secret_api_key").IsRequired();
            entity.Property(e => e.PublicJwtKey).HasColumnName("public_jwt_key")
                        .HasConversion(
                            pemKey => pemKey == null ? null : pemKey.Inner(),
                            str => !string.IsNullOrEmpty(str) ? new PEMKey(str) : null);

            // Configure JSON properties
            entity.Property(e => e.Settings)
                .HasColumnName("settings")
                .HasColumnType("jsonb")
                .HasConversion(
                    v => JsonSerializer.SerializeToDocument(v, (JsonSerializerOptions?)null),
                    v => v.Deserialize<AccountSettings>((JsonSerializerOptions?)null) ?? new AccountSettings());

            // Ignore metadata property since it's not in the database schema
            entity.Ignore(e => e.Metadata);

            // Indexes
            entity.HasIndex(e => e.SecretApiKey).IsUnique();
        });

        // Configure User entity
        modelBuilder.Entity<User>(entity =>
        {
            entity.ToTable("users");
            entity.HasKey(e => e.Id);
            entity.Property(e => e.Id).HasColumnName("user_uid").HasConversion(
                id => (Guid)id,
                guid => (Id)guid);
            entity.Property(e => e.AccountId).HasColumnName("account_uid").HasConversion(
                id => (Guid)id,
                guid => (Id)guid);
            entity.Property(e => e.ExternalId).HasColumnName("external_id");

            // Configure JSON properties
            entity.Property(e => e.Metadata)
                .HasColumnName("metadata")
                .HasColumnType("jsonb")
                .HasConversion(
                    v => JsonSerializer.SerializeToDocument(v, (JsonSerializerOptions?)null),
                    v => v.Deserialize<Metadata>((JsonSerializerOptions?)null) ?? new Metadata());

            // Ignore properties that don't exist in the database schema
            entity.Ignore(e => e.Name);
            entity.Ignore(e => e.Email);

            // Relationships
            entity.HasOne<Account>()
                .WithMany()
                .HasForeignKey(e => e.AccountId)
                .OnDelete(DeleteBehavior.Cascade);

            // Indexes
            entity.HasIndex(e => new { e.AccountId, e.ExternalId }).IsUnique();
        });

        // Configure UserIntegration entity
        modelBuilder.Entity<UserIntegration>(entity =>
        {
            entity.ToTable("user_integrations");
            entity.HasKey(e => new { e.UserId, e.Provider });
            entity.Property(e => e.UserId).HasColumnName("user_uid").HasConversion(
                id => (Guid)id,
                guid => (Id)guid);
            entity.Property(e => e.AccountId).HasColumnName("account_uid").HasConversion(
                id => (Guid)id,
                guid => (Id)guid);
            entity.Property(e => e.Provider).HasColumnName("provider");
            entity.Property(e => e.RefreshToken).HasColumnName("refresh_token").IsRequired();
            entity.Property(e => e.AccessToken).HasColumnName("access_token").IsRequired();
            entity.Property(e => e.AccessTokenExpiresTs).HasColumnName("access_token_expires_ts");

            // Relationships
            entity.HasOne<User>()
                .WithMany()
                .HasForeignKey(e => e.UserId)
                .OnDelete(DeleteBehavior.Cascade);

            entity.HasOne<Account>()
                .WithMany()
                .HasForeignKey(e => e.AccountId)
                .OnDelete(DeleteBehavior.Cascade);

            // Indexes
            entity.HasIndex(e => new { e.UserId, e.Provider }).IsUnique();
        });

        // Configure Calendar entity
        modelBuilder.Entity<Calendar>(entity =>
        {
            entity.ToTable("calendars");
            entity.HasKey(e => e.Id);
            entity.Property(e => e.Id).HasColumnName("calendar_uid").HasConversion(
                id => (Guid)id,
                guid => (Id)guid);
            entity.Property(e => e.UserId).HasColumnName("user_uid").HasConversion(
                id => (Guid)id,
                guid => (Id)guid);
            entity.Property(e => e.AccountId).HasColumnName("account_uid").HasConversion(
                id => (Guid)id,
                guid => (Id)guid);
            entity.Property(e => e.Key).HasColumnName("key");
            entity.Property(e => e.Name).HasColumnName("name");

            // Configure JSON properties
            entity.Property(e => e.Settings)
                .HasColumnName("settings")
                .HasColumnType("jsonb")
                .HasConversion(
                    v => JsonSerializer.SerializeToDocument(v, (JsonSerializerOptions?)null),
                    v => v.Deserialize<CalendarSettings>((JsonSerializerOptions?)null) ?? new CalendarSettings());

            entity.Property(e => e.Metadata)
                .HasColumnName("metadata")
                .HasColumnType("jsonb")
                .HasConversion(
                    v => JsonSerializer.SerializeToDocument(v, (JsonSerializerOptions?)null),
                    v => v.Deserialize<Metadata>((JsonSerializerOptions?)null) ?? new Metadata());

            // Relationships
            entity.HasOne<User>()
                .WithMany()
                .HasForeignKey(e => e.UserId)
                .OnDelete(DeleteBehavior.Cascade);

            entity.HasOne<Account>()
                .WithMany()
                .HasForeignKey(e => e.AccountId)
                .OnDelete(DeleteBehavior.Cascade);

            // Indexes
            entity.HasIndex(e => e.UserId);
        });

        // Configure CalendarEvent entity
        modelBuilder.Entity<CalendarEvent>(entity =>
        {
            entity.ToTable("calendar_events");
            entity.HasKey(e => e.Id);
            entity.Property(e => e.Id).HasColumnName("event_uid").HasConversion(
                id => (Guid)id,
                guid => (Id)guid);
            entity.Property(e => e.CalendarId).HasColumnName("calendar_uid").HasConversion(
                id => (Guid)id,
                guid => (Id)guid);
            entity.Property(e => e.UserId).HasColumnName("user_uid").HasConversion(
                id => (Guid)id,
                guid => (Id)guid);
            entity.Property(e => e.AccountId).HasColumnName("account_uid").HasConversion(
                id => (Guid)id,
                guid => (Id)guid);
            entity.Property(e => e.ExternalParentId).HasColumnName("external_parent_id");
            entity.Property(e => e.Title).HasColumnName("title");
            entity.Property(e => e.Description).HasColumnName("description");
            entity.Property(e => e.Location).HasColumnName("location");
            entity.Property(e => e.EventType).HasColumnName("event_type");
            entity.Property(e => e.AllDay).HasColumnName("all_day").IsRequired();
            entity.Property(e => e.Status).HasColumnName("status").HasConversion<string>();
            entity.Property(e => e.StartTime).HasColumnName("start_time").IsRequired();
            entity.Property(e => e.Duration).HasColumnName("duration").IsRequired();
            entity.Property(e => e.EndTime).HasColumnName("end_time").IsRequired();
            entity.Property(e => e.Busy).HasColumnName("busy").IsRequired();
            entity.Property(e => e.Created).HasColumnName("created").IsRequired()
                .HasConversion(
                    v => ((DateTimeOffset)v).ToUnixTimeMilliseconds(),
                    v => DateTimeOffset.FromUnixTimeMilliseconds(v).DateTime);
            entity.Property(e => e.Updated).HasColumnName("updated").IsRequired()
                .HasConversion(
                    v => ((DateTimeOffset)v).ToUnixTimeMilliseconds(),
                    v => DateTimeOffset.FromUnixTimeMilliseconds(v).DateTime);
            entity.Property(e => e.ExternalId).HasColumnName("external_id");
            entity.Property(e => e.RecurringEventId).HasColumnName("recurring_event_uid").HasConversion(
                id => id.HasValue ? (Guid?)id.Value : null,
                guid => guid.HasValue ? (Id?)guid.Value : null);
            entity.Property(e => e.OriginalStartTime).HasColumnName("original_start_time");
            entity.Property(e => e.RecurringUntil).HasColumnName("recurring_until");
            entity.Property(e => e.ServiceId).HasColumnName("service_uid").HasConversion(
                id => id.HasValue ? (Guid?)id.Value : null,
                guid => guid.HasValue ? (Id?)guid.Value : null);

            // Configure JSON properties
            entity.Property(e => e.Recurrence)
                .HasColumnName("recurrence_jsonb")
                .HasColumnType("jsonb")
                .HasConversion(
                    v => v == null ? null : JsonSerializer.SerializeToDocument(v, (JsonSerializerOptions?)null),
                    v => v == null ? null : v.Deserialize<RRuleOptions>((JsonSerializerOptions?)null));

            entity.Property(e => e.ExDates)
                .HasColumnName("exdates")
                .HasColumnType("timestamp with time zone[]")
                .HasConversion(
                    v => v.Select(dt => (DateTimeOffset)dt).ToArray(),
                    v => v.Select(dto => dto.DateTime).ToList());

            entity.Property(e => e.Metadata)
                .HasColumnName("metadata")
                .HasColumnType("jsonb")
                .HasConversion(
                    v => JsonSerializer.SerializeToDocument(v, (JsonSerializerOptions?)null),
                    v => v.Deserialize<Metadata>((JsonSerializerOptions?)null) ?? new Metadata());

            entity.Property(e => e.Reminders)
                .HasColumnName("reminders_jsonb")
                .HasColumnType("jsonb")
                .HasConversion(
                    v => JsonSerializer.SerializeToDocument(v, (JsonSerializerOptions?)null),
                    v => v.Deserialize<List<CalendarEventReminder>>((JsonSerializerOptions?)null) ?? new List<CalendarEventReminder>());

            // Relationships
            entity.HasOne<Calendar>()
                .WithMany()
                .HasForeignKey(e => e.CalendarId)
                .OnDelete(DeleteBehavior.Cascade);

            entity.HasOne<User>()
                .WithMany()
                .HasForeignKey(e => e.UserId)
                .OnDelete(DeleteBehavior.Cascade);

            entity.HasOne<Account>()
                .WithMany()
                .HasForeignKey(e => e.AccountId)
                .OnDelete(DeleteBehavior.Cascade);

            // Indexes
            entity.HasIndex(e => e.CalendarId);
            entity.HasIndex(e => e.ExternalId);
            entity.HasIndex(e => e.StartTime);
            entity.HasIndex(e => e.EndTime);
            entity.HasIndex(e => e.RecurringEventId);
            entity.HasIndex(e => new { e.CalendarId, e.ExternalId }).IsUnique();
        });

        // Configure Service entity
        modelBuilder.Entity<Service>(entity =>
        {
            entity.ToTable("services");
            entity.HasKey(e => e.Id);
            entity.Property(e => e.Id).HasColumnName("service_uid").HasConversion(
                id => (Guid)id,
                guid => (Id)guid);
            entity.Property(e => e.AccountId).HasColumnName("account_uid").HasConversion(
                id => (Guid)id,
                guid => (Id)guid);

            // Configure JSON properties
            entity.Property(e => e.MultiPerson)
                .HasColumnName("multi_person")
                .HasColumnType("jsonb")
                .HasConversion(
                    v => JsonSerializer.SerializeToDocument(v, (JsonSerializerOptions?)null),
                    v => v.Deserialize<ServiceMultiPersonOptions>((JsonSerializerOptions?)null) ?? new ServiceMultiPersonOptions());

            entity.Property(e => e.Metadata)
                .HasColumnName("metadata")
                .HasColumnType("jsonb")
                .HasConversion(
                    v => JsonSerializer.SerializeToDocument(v, (JsonSerializerOptions?)null),
                    v => v.Deserialize<Metadata>((JsonSerializerOptions?)null) ?? new Metadata());

            // Relationships
            entity.HasOne<Account>()
                .WithMany()
                .HasForeignKey(e => e.AccountId)
                .OnDelete(DeleteBehavior.Cascade);

            // Indexes
            entity.HasIndex(e => e.AccountId);
        });

        // Configure Schedule entity
        modelBuilder.Entity<Schedule>(entity =>
        {
            entity.ToTable("schedules");
            entity.HasKey(e => e.Id);
            entity.Property(e => e.Id).HasColumnName("schedule_uid").HasConversion(
                id => (Guid)id,
                guid => (Id)guid);
            entity.Property(e => e.UserId).HasColumnName("user_uid").HasConversion(
                id => (Guid)id,
                guid => (Id)guid);

            // Configure JSON properties
            entity.Property(e => e.Rules)
                .HasColumnName("rules")
                .HasColumnType("jsonb")
                .HasConversion(
                    v => JsonSerializer.SerializeToDocument(v, (JsonSerializerOptions?)null),
                    v => v.Deserialize<List<ScheduleRule>>((JsonSerializerOptions?)null) ?? new List<ScheduleRule>());

            entity.Property(e => e.Metadata)
                .HasColumnName("metadata")
                .HasColumnType("jsonb")
                .HasConversion(
                    v => JsonSerializer.SerializeToDocument(v, (JsonSerializerOptions?)null),
                    v => v.Deserialize<Metadata>((JsonSerializerOptions?)null) ?? new Metadata());

            // Ignore properties that should be stored in metadata
            entity.Ignore(e => e.AccountId);
            entity.Ignore(e => e.Name);

            // Relationships
            entity.HasOne<User>()
                .WithMany()
                .HasForeignKey(e => e.UserId)
                .OnDelete(DeleteBehavior.Cascade);

            // Indexes
            entity.HasIndex(e => e.UserId);
        });



        // Configure Reminder entity
        modelBuilder.Entity<Reminder>(entity =>
        {
            entity.ToTable("reminders");
            entity.HasKey(e => e.Id);
            entity.Property(e => e.Id).HasColumnName("id").HasConversion(
                id => (Guid)id,
                guid => (Id)guid);
            entity.Property(e => e.EventId).HasColumnName("event_uid").HasConversion(
                id => (Guid)id,
                guid => (Id)guid);
            entity.Property(e => e.AccountId).HasColumnName("account_uid").HasConversion(
                id => (Guid)id,
                guid => (Id)guid);
            entity.Property(e => e.RemindAt).HasColumnName("remind_at").IsRequired();
            entity.Property(e => e.Version).HasColumnName("version").IsRequired();
            entity.Property(e => e.Identifier).HasColumnName("identifier").IsRequired();
            entity.Property(e => e.Type).HasColumnName("type").HasConversion<string>();
            entity.Property(e => e.Time).HasColumnName("time").IsRequired();
            entity.Property(e => e.Sent).HasColumnName("sent").IsRequired();

            // Relationships
            entity.HasOne<CalendarEvent>()
                .WithMany()
                .HasForeignKey(e => e.EventId)
                .OnDelete(DeleteBehavior.Cascade);

            entity.HasOne<Account>()
                .WithMany()
                .HasForeignKey(e => e.AccountId)
                .OnDelete(DeleteBehavior.Cascade);

            // Indexes
            entity.HasIndex(e => e.EventId);
            entity.HasIndex(e => e.Time);
            entity.HasIndex(e => e.Sent);
        });
    }
}