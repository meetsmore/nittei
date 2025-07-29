namespace Nittei.Domain;

/// <summary>
/// Date utility functions
/// </summary>
public static class Date
{
  /// <summary>
  /// Validates a date string and returns the components
  /// </summary>
  public static (int year, uint month, uint day) IsValidDate(string dateStr)
  {
    var dates = dateStr.Split('-');
    if (dates.Length != 3)
      throw new ArgumentException(dateStr, nameof(dateStr));

    if (!int.TryParse(dates[0], out var year) ||
        !uint.TryParse(dates[1], out var month) ||
        !uint.TryParse(dates[2], out var day))
    {
      throw new ArgumentException(dateStr, nameof(dateStr));
    }

    if (year < 1970 || year > 2100 || month < 1 || month > 12)
      throw new ArgumentException(dateStr, nameof(dateStr));

    var monthLength = GetMonthLength(year, (int)month);
    if (day < 1 || day > monthLength)
      throw new ArgumentException(dateStr, nameof(dateStr));

    return (year, month, day);
  }

  /// <summary>
  /// Checks if a year is a leap year
  /// </summary>
  public static bool IsLeapYear(int year)
  {
    return year % 400 == 0 || (year % 100 != 0 && year % 4 == 0);
  }

  /// <summary>
  /// Gets the length of a month (1-based)
  /// </summary>
  public static uint GetMonthLength(int year, int month)
  {
    return month switch
    {
      1 => 31,
      2 => IsLeapYear(year) ? (uint)29 : (uint)28,
      3 => 31,
      4 => 30,
      5 => 31,
      6 => 30,
      7 => 31,
      8 => 31,
      9 => 30,
      10 => 31,
      11 => 30,
      12 => 31,
      _ => throw new ArgumentException($"Invalid month: {month}", nameof(month))
    };
  }

  /// <summary>
  /// Formats a date as YYYY-M-D
  /// </summary>
  public static string FormatDate(DateTime date)
  {
    return $"{date.Year}-{date.Month}-{date.Day}";
  }
}