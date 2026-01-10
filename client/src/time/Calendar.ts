/**
 * Calendar module for game time calculations.
 * 
 * Custom calendar system:
 * - 360 days per year
 * - 12 months of 30 days each
 * - 24 hours per day
 * - 60 minutes per hour
 */

// Custom calendar constants
// 360 days per year, 12 months of 30 days each, 24 hours per day
const MINUTES_PER_HOUR = 60;
const HOURS_PER_DAY = 24;
const MINUTES_PER_DAY = HOURS_PER_DAY * MINUTES_PER_HOUR; // 1440
const DAYS_PER_MONTH = 30;
const MONTHS_PER_YEAR = 12;
const DAYS_PER_YEAR = DAYS_PER_MONTH * MONTHS_PER_YEAR; // 360
const MINUTES_PER_YEAR = DAYS_PER_YEAR * MINUTES_PER_DAY; // 518,400

/**
 * Calendar date structure.
 */
export interface CalendarDate {
    /** Year number */
    year: number;
    /** Month number (1-12) */
    month: number;
    /** Day number (1-30) */
    day: number;
    /** Hour (0-23) */
    hour: number;
    /** Minute (0-59) */
    minute: number;
}

/**
 * Calendar utility for the game's custom calendar system.
 * 
 * Calendar: 360 days/year, 12 months of 30 days, 24-hour days.
 * Provides date/time conversion and formatting functions.
 */
export class GameCalendar {
    /**
     * Convert total game minutes to custom calendar date/time.
     * Calendar: 360 days/year, 12 months of 30 days, 24-hour days.
     */
    public static getCalendarDate(totalMinutes: number): CalendarDate {
        const year = Math.floor(totalMinutes / MINUTES_PER_YEAR);
        const minutesInYear = totalMinutes % MINUTES_PER_YEAR;
        const dayOfYear = Math.floor(minutesInYear / MINUTES_PER_DAY);
        const month = Math.floor(dayOfYear / DAYS_PER_MONTH) + 1; // 1-12
        const day = (dayOfYear % DAYS_PER_MONTH) + 1; // 1-30
        const minutesInDay = totalMinutes % MINUTES_PER_DAY;
        const hour = Math.floor(minutesInDay / MINUTES_PER_HOUR);
        const minute = minutesInDay % MINUTES_PER_HOUR;
        
        return { year, month, day, hour, minute };
    }

    /**
     * Format date/time as YYYY/MM/DD HH:MM
     */
    public static formatDateTime(totalMinutes: number): string {
        const { year, month, day, hour, minute } = this.getCalendarDate(totalMinutes);
        const yearStr = year.toString().padStart(4, '0');
        const monthStr = month.toString().padStart(2, '0');
        const dayStr = day.toString().padStart(2, '0');
        const hourStr = hour.toString().padStart(2, '0');
        const minuteStr = minute.toString().padStart(2, '0');
        return `${yearStr}/${monthStr}/${dayStr} ${hourStr}:${minuteStr}`;
    }

    /**
     * Get time of day in hours (0-24) for celestial/lighting calculations
     */
    public static getTimeOfDayHours(totalMinutes: number): number {
        const minutesInDay = totalMinutes % MINUTES_PER_DAY;
        return minutesInDay / MINUTES_PER_HOUR;
    }
}


