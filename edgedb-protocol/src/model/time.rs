use crate::model::OutOfRangeError;
use std::{convert::TryFrom, time::SystemTime, fmt::{Debug, Display}};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Duration {
    pub(crate) micros: i64,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LocalDatetime {
    pub(crate) micros: i64,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LocalDate {
    pub(crate) days: i32,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LocalTime {
    pub(crate) micros: u64,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Datetime {
    pub(crate) micros: i64,
}

impl Duration {
    pub fn from_micros(micros: i64) -> Duration {
        Duration { micros }
    }

    pub fn to_micros(self) -> i64 {
        self.micros
    }

    // Returns true if self is positive and false if the duration
    // is zero or negative.
    pub fn is_positive(&self) -> bool {
        self.micros.is_positive()
    }
    // Returns true if self is negative and false if the duration
    // is zero or positive.
    pub fn is_negative(&self) -> bool {
        self.micros.is_negative()
    }
    // Returns absolute values as stdlib's duration
    //
    // Note: `std::time::Duration` can't be negative
    pub fn abs_duration(&self) -> std::time::Duration {
        if self.micros.is_negative() {
            return std::time::Duration::from_micros(
                u64::MAX - self.micros as u64 + 1);
        } else {
            return std::time::Duration::from_micros(self.micros as u64);
        }
    }
}

const MICROS_PER_DAY : u64 = 86_400 * 1_000_000;

impl LocalDatetime {
    pub fn from_micros(micros: i64) -> LocalDatetime {
        return LocalDatetime { micros }
    }

    pub fn to_micros(self) -> i64 {
        self.micros
    }

    pub fn date(self) -> LocalDate {
        LocalDate::from_days(self.micros.wrapping_div_euclid(MICROS_PER_DAY as i64) as i32)
    }

    pub fn time(self) -> LocalTime {
        LocalTime::from_micros(self.micros.wrapping_rem_euclid(MICROS_PER_DAY as i64) as u64)
    }
}

impl Display for LocalDatetime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.date(), self.time())
    }
}

impl Debug for LocalDatetime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}T{}", self.date(), self.time())
    }
}

impl LocalTime {
    pub const MIDNIGHT : LocalTime = LocalTime { micros: 0 };
    pub const MAX : LocalTime = LocalTime { micros: MICROS_PER_DAY - 1 };

    pub(crate) fn try_from_micros(micros: u64) -> Result<LocalTime, OutOfRangeError> {
        if micros < MICROS_PER_DAY { 
            Ok(LocalTime { micros: micros }) 
        } else {
             Err(OutOfRangeError) 
        }
    }

    pub fn from_micros(micros: u64) -> LocalTime {
        Self::try_from_micros(micros).ok().expect("LocalTime is out of range")
    }

    pub fn to_micros(self) -> u64 {
        self.micros
    }

    fn parts(self) -> (u8, u8, u8, u32) {
        let micros = self.micros;

        let microsecond = (micros % 1_000_000) as u32;
        let micros = micros / 1_000_000;

        let second = (micros % 60) as u8;
        let micros = micros / 60;

        let minute = (micros % 60) as u8;
        let micros = micros / 60;

        let hour = (micros % 24) as u8;
        let micros = micros / 24;
        debug_assert_eq!(0, micros);

        (hour, minute, second, microsecond)
    }
}

impl Display for LocalTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}

impl Debug for LocalTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (hour, minute, second, microsecond) = self.parts();
        write!(f, "{:02}:{:02}:{:02}", hour, minute, second)?;
        // like chrono::NaiveTime it outputs either 0, 3 or 6 decimal digits
        if microsecond != 0 {
            if microsecond % 1000 == 0 {
                write!(f, ".{:03}", microsecond / 1000)?;
            } else {
                write!(f, ".{:06}", microsecond)?;
            }
        };
        Ok(())
    }
}


const DAYS_IN_400_YEARS : u32 = 400 * 365 + 97;
const DAYS_IN_100_YEARS : u32 = 100 * 365 + 24;
const DAYS_IN_4_YEARS :u32 = 4 * 365 + 1;
const DAYS_IN_1_YEAR : u32 = 365;

const DAY_TO_MONTH_365 : [u32; 13] = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334, 365];
const DAY_TO_MONTH_366 : [u32; 13] = [0, 31, 60, 91, 121, 152, 182, 213, 244, 274, 305, 335, 366];

impl LocalDate {
    pub const MIN : LocalDate = LocalDate { days :-14712 * 365 }; //todo: figure out value
    pub const MAX : LocalDate = LocalDate { days: i32::MAX }; //todo: figure out value

    pub fn from_days(days: i32) -> LocalDate {
        assert!(days >= Self::MIN.days);
        return LocalDate { days }
    }

    pub fn to_days(self) -> i32 {
        self.days
    }

    pub fn from_ymd(year:i32, month: u8, day:u8) -> LocalDate {
        Self::try_from_ymd(year, month, day).expect("invalid date")
    }

    fn try_from_ymd(year:i32, month: u8, day:u8) -> Result<LocalDate, OutOfRangeError> {
        if day < 1 || day > 31 {
            return Err(OutOfRangeError);
        }
        if month < 1 || month > 12 {
            return Err(OutOfRangeError);
        }
        if year < -4712 || year > 5874897 {
           return Err(OutOfRangeError);
        }

        let passed_years = (year + 4800 - 1) as u32; // year -4800 is smaller than MIN
        let days_from_year =
            365 * passed_years
            + passed_years / 4 
            - passed_years / 100
            + passed_years / 400
            + 366;

        let is_leap_year = (year % 400 == 0) || (year % 4 == 0 && year % 100 != 0);
        let day_to_month =
            if is_leap_year { DAY_TO_MONTH_366 } else { DAY_TO_MONTH_365 };

        let day_in_year = (day - 1) as u32 + day_to_month[month as usize - 1];
        if day_in_year >= day_to_month[month as usize] {
            return Err(OutOfRangeError);
        }

        Ok(LocalDate::from_days((days_from_year + day_in_year) as i32
         - DAYS_IN_400_YEARS as i32 * 17))
    }

    fn to_ymd(self) -> (i32, u8, u8) {
        const DAY_TO_MONTH_MARCH : [u32; 12] = [0, 31, 61, 92, 122, 153, 184, 214, 245, 275, 306, 337];
        const MARCH_1 : u32 = 31 + 29;
        const MARCH_1_MINUS_4800_TO_POSTGRES_EPOCH : u32 = 17 * DAYS_IN_400_YEARS - MARCH_1;

        let days = (self.days as u32).wrapping_add(MARCH_1_MINUS_4800_TO_POSTGRES_EPOCH);

        let years400 = days / DAYS_IN_400_YEARS;
        let days = days % DAYS_IN_400_YEARS;

        let mut years100 = days / DAYS_IN_100_YEARS;
        if years100 == 4 { years100 = 3 }; // prevent 400 year leap day from overflowing
        let days = days - DAYS_IN_100_YEARS * years100;

        let years4 = days / DAYS_IN_4_YEARS;
        let days = days % DAYS_IN_4_YEARS;

        let mut years1 = days / DAYS_IN_1_YEAR;
        if years1 == 4 { years1 = 3 }; // prevent 4 year leap day from overflowing
        let days = days - DAYS_IN_1_YEAR * years1;

        let years = years1 + years4 * 4 + years100 * 100 + years400 * 400;
        let month_entry = DAY_TO_MONTH_MARCH.iter().filter(|d| days >= **d).enumerate().last().unwrap();
        let months = years * 12 + 2 + month_entry.0 as u32;
        let year = (months / 12) as i32 - 4800;
        let month = (months % 12 + 1) as u8;
        let day = (days - month_entry.1 + 1) as u8;

        (year, month, day)
    }
}

impl Display for LocalDate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}

impl Debug for LocalDate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (year, month, day) = self.to_ymd();
        if year > 10_000 { // ISO format requires a + on dates longer than 4 digits
            write!(f, "+")?;
        }
        if year >= 0 {
            write!(f, "{:04}-{:02}-{:02}", year, month, day)
        } else {
            write!(f, "{:05}-{:02}-{:02}", year, month, day)
        }
    }
}

impl Datetime {
    pub fn from_micros(micros: i64) -> Datetime {
        Datetime { micros }
    }

    pub fn to_micros(self) -> i64 {
        self.micros
    }

    fn to_system_time(self) -> SystemTime {
        use std::time::{ Duration, UNIX_EPOCH };
        // postgres epoch starts at 2020-01-01
        let postgres_epoch :SystemTime = UNIX_EPOCH + Duration::from_secs(10957 * 86_400);
       
        if self.micros > 0 {
            postgres_epoch + Duration::from_micros(self.micros as u64)
        } else {
            postgres_epoch - Duration::from_micros((-self.micros) as u64)
        }
    }

    fn from_system_time(time:SystemTime) -> Result<Datetime, OutOfRangeError> {
        let min_system_time = Datetime::from_micros(i64::min_value()).to_system_time();
        let duration = time.duration_since(min_system_time).map_err(|_| OutOfRangeError)?;
        let micros = duration.as_micros();
        if micros > u64::max_value() as u128 {
            return Err(OutOfRangeError);
        }
        let micros = (micros + i64::min_value() as u128) as i64;
        Ok(Datetime::from_micros(micros))
    }
}

impl Display for Datetime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} UTC", LocalDatetime::from_micros(self.to_micros()))
    }
}

impl Debug for Datetime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}Z", LocalDatetime::from_micros(self.to_micros()))
    }
}

impl From<Datetime> for SystemTime {
    fn from(value: Datetime) -> Self {
        Datetime::to_system_time(value)
    }
}

impl TryFrom<SystemTime> for Datetime {
    type Error = OutOfRangeError;

    fn try_from(value: SystemTime) -> Result<Self, Self::Error> {
        Datetime::from_system_time(value)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn big_duration_abs() {
        use super::Duration as Src;
        use std::time::Duration as Trg;
        assert_eq!(Src { micros: -1 }.abs_duration(), Trg::new(0, 1000));
        assert_eq!(Src { micros: -1000 }.abs_duration(), Trg::new(0, 1000000));
        assert_eq!(Src { micros: -1000000 }.abs_duration(), Trg::new(1, 0));
        assert_eq!(
            Src {
                micros: i64::min_value()
            }
            .abs_duration(),
            Trg::new(9223372036854, 775808000)
        );
    }

    #[test]
    fn local_date_from_ymd() {
        assert_eq!(0, LocalDate::from_ymd(2000, 1, 1).to_days());
        assert_eq!(-365, LocalDate::from_ymd(1999, 1, 1).to_days());
        assert_eq!(366, LocalDate::from_ymd(2001, 1, 1).to_days());
        assert_eq!(-730119, LocalDate::from_ymd(0001, 1, 1).to_days());
        assert_eq!(2921575, LocalDate::from_ymd(9999, 1, 1).to_days());

        assert_eq!(Err(OutOfRangeError), LocalDate::try_from_ymd(2001, 1, 32));
        assert_eq!(Err(OutOfRangeError), LocalDate::try_from_ymd(2001, 2, 29));
    }

    #[test]
    fn local_date_from_ymd_leap_year() {
        let days_in_month_leap = [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
        let mut total_days = 0;
        let start_of_year = 365 * 4 + 1;
        for month in 1..=12 {
            let start_of_month = LocalDate::from_ymd(2004, month as u8, 1).to_days();
            assert_eq!(total_days, start_of_month - start_of_year);

            let days_in_current_month = days_in_month_leap[month - 1];
            total_days += days_in_current_month;

            let end_of_month = LocalDate::from_ymd(2004, month as u8, days_in_current_month as u8).to_days();
            assert_eq!(total_days - 1, end_of_month - start_of_year);
        }
        assert_eq!(366, total_days);
    }

    const DAYS_IN_MONTH_LEAP :[u8; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

    #[test]
    fn local_date_from_ymd_normal_year() {
        let mut total_days = 0;
        let start_of_year = 365 + 1;
        for month in 1..=12 {
            let start_of_month = LocalDate::from_ymd(2001, month as u8, 1).to_days();
            assert_eq!(total_days, start_of_month - start_of_year);

            let days_in_current_month = DAYS_IN_MONTH_LEAP[month - 1];
            total_days += days_in_current_month as i32;

            let end_of_month = LocalDate::from_ymd(2001, month as u8, days_in_current_month as u8).to_days();
            assert_eq!(total_days - 1, end_of_month - start_of_year);
        }
        assert_eq!(365, total_days);
    }

    fn extended_test_dates() -> impl Iterator<Item=(i32, u8, u8)> {
        const YEARS :[i32; 27]= [
            -4700,
            -4400,
            -1,
            0,
            1,
            2,
            1000,
            1999,
            2000,
            2001,
            2002,
            2003,
            2004,
            2008,
            2009,
            2010,
            2100,
            2200,
            2300,
            2400,
            9000,
            9999,
            10000,
            10001,
            11000,
            20000,
            100000,
        ];

        const MONTHS : std::ops::RangeInclusive<u8>= 1u8..=12;
        const DAYS :[u8; 6] = [1u8, 13, 28, 29, 30, 31];
        let dates = MONTHS
            .flat_map(|month| DAYS.iter().map(move |day| (month, *day)));

        YEARS
            .iter()
            .flat_map(move|year| dates.clone().map(move |date| (*year, date.0, date.1)))
    }

    fn valid_test_dates() -> impl Iterator<Item=(i32, u8, u8)> {
        extended_test_dates().filter(|date| 
                LocalDate::try_from_ymd(date.0, date.1, date.2).is_ok())
    }

    #[test]
    fn check_test_dates() {
        assert!(valid_test_dates().count() > 1000);
    }


    #[test]
    fn local_date_ymd_roundtrip() {
        for (year, month, day) in valid_test_dates() {
            assert_eq!((year, month, day), LocalDate::from_ymd(year, month, day).to_ymd());
        }
    }

    #[test]
    fn format_local_date() {
        const DAYS_IN_2000_YEARS : i32 = 730485;
        assert_eq!("2000-01-01", LocalDate::from_days(0).to_string());
        assert_eq!("0000-01-01", LocalDate::from_days(-DAYS_IN_2000_YEARS).to_string());
        assert_eq!("0001-01-01", LocalDate::from_days(-DAYS_IN_2000_YEARS + 366).to_string());
        assert_eq!("-0001-01-01", LocalDate::from_days(-DAYS_IN_2000_YEARS - 365).to_string());
        assert_eq!("-4000-01-01", LocalDate::from_days(-3 * DAYS_IN_2000_YEARS).to_string());
        assert_eq!("+10001-01-01", LocalDate::from_days(4 * DAYS_IN_2000_YEARS + 366).to_string());
    }

    #[test]
    fn format_local_time() {
        assert_eq!("00:00:00", LocalTime::MIDNIGHT.to_string());
        assert_eq!("00:00:00.010", LocalTime::from_micros(10_000).to_string());
        assert_eq!("00:00:00.010020", LocalTime::from_micros(10_020).to_string());
        assert_eq!("23:59:59.999999", LocalTime::MAX.to_string());
    }

    fn to_debug<T:Debug>(x:T) -> String {
        format!("{:?}", x)
    }

    #[test]
    fn format_local_datetime() {
        assert_eq!("2039-02-13 23:31:30.123456", LocalDatetime::from_micros(1_234_567_890_123_456).to_string());
        assert_eq!("2039-02-13T23:31:30.123456", to_debug(LocalDatetime::from_micros(1_234_567_890_123_456)));
    }

    #[test]
    fn format_datetime() {
        assert_eq!("2039-02-13 23:31:30.123456 UTC", Datetime::from_micros(1_234_567_890_123_456).to_string());
        assert_eq!("2039-02-13T23:31:30.123456Z", to_debug(Datetime::from_micros(1_234_567_890_123_456)));
    }
}

#[cfg(feature = "chrono")]
mod chrono_interop {
    use super::{LocalDate, LocalDatetime, LocalTime};
    use chrono::naive::{NaiveDate, NaiveDateTime, NaiveTime};

    impl std::convert::TryInto<NaiveDateTime> for &LocalDatetime {
        type Error = OutOfRangeError;
        fn try_into(self) -> Result<NaiveDateTime, Self::Error> {
            NaiveDateTime::from_timestamp_opt(self.micros/1000_000,
                ((self.micros % 1000_000)*1000) as u32)
            .ok_or(OutOfRangeError)
        }
    }

    impl std::convert::TryFrom<&NaiveDateTime> for LocalDatetime {
        type Error = OutOfRangeError;
        fn try_from(d: &NaiveDateTime)
            -> Result<LocalDatetime, Self::Error>
        {
            let secs = d.timestamp();
            let micros = d.timestamp_subsec_micros();
            Ok(LocalDatetime {
                micros: secs.checked_mul(1_000_000)
                    .and_then(|x| x.checked_add(micros as i64))
                    .ok_or(OutOfRangeError)?,
            })
        }
    }

    impl std::convert::TryFrom<&NaiveDate> for LocalDate {
        type Error = OutOfRangeError;
        fn try_from(d: &NaiveDate) -> Result<LocalDate, Self::Error>
        {
            let days = chrono::Datelike::num_days_from_ce(d);
            Ok(LocalDate {
                days: days.checked_sub(730120)
                    .ok_or(OutOfRangeError)?,
            })
        }
    }

    impl std::convert::TryInto<NaiveDate> for &LocalDate {
        type Error = OutOfRangeError;
        fn try_into(self) -> Result<NaiveDate, Self::Error> {
            self.days.checked_add(730120)
            .and_then(NaiveDate::from_num_days_from_ce_opt)
            .ok_or(OutOfRangeError)
        }
    }

    impl Into<NaiveTime> for &LocalTime {
        fn into(self) -> NaiveTime {
            NaiveTime::from_num_seconds_from_midnight(
                (self.micros / 1000_000) as u32,
                ((self.micros % 1000_000) * 1000) as u32)
        }
    }

    impl From<&NaiveTime> for LocalTime {
        fn from(time: &NaiveTime) -> LocalTime {
            let sec = chrono::Timelike::num_seconds_from_midnight(time);
            let nanos = chrono::Timelike::nanosecond(time);
            LocalTime {
                micros: sec as i64 * 1000_000 + nanos as i64 / 1000,
            }
        }
    }

    impl std::convert::TryInto<NaiveDateTime> for LocalDatetime {
        type Error = OutOfRangeError;
        fn try_into(self) -> Result<NaiveDateTime, Self::Error> {
            (&self).try_into()
        }
    }

    impl std::convert::TryInto<NaiveDate> for LocalDate {
        type Error = OutOfRangeError;
        fn try_into(self) -> Result<NaiveDate, Self::Error> {
            (&self).try_into()
        }
    }

    impl std::convert::TryFrom<NaiveDate> for LocalDate {
        type Error = OutOfRangeError;
        fn try_from(d: NaiveDate) -> Result<LocalDate, Self::Error>
        {
            std::convert::TryFrom::try_from(&d)
        }
    }

    impl Into<NaiveTime> for LocalTime {
        fn into(self) -> NaiveTime {
            (&self).into()
        }
    }

    impl std::convert::TryFrom<NaiveDateTime> for LocalDatetime {
        type Error = OutOfRangeError;
        fn try_from(d: NaiveDateTime)
            -> Result<LocalDatetime, Self::Error>
        {
            std::convert::TryFrom::try_from(&d)
        }
    }

    impl From<NaiveTime> for LocalTime {
        fn from(time: NaiveTime) -> LocalTime {
            From::from(&time)
        }
    }

    #[cfg(test)]
    mod test {
        use super::*;
        use std::convert::{TryFrom, TryInto};
        use std::str::FromStr;

        #[test]
        fn chrono_roundtrips() -> Result<(), Box<dyn std::error::Error>> {
            let naive = NaiveDateTime::from_str("2019-12-27T01:02:03.123456")?;
            assert_eq!(naive,
                TryInto::<NaiveDateTime>::try_into(
                    LocalDatetime::try_from(naive)?)?);
            let naive = NaiveDate::from_str("2019-12-27")?;
            assert_eq!(naive,
                TryInto::<NaiveDate>::try_into(LocalDate::try_from(naive)?)?);
            let naive = NaiveTime::from_str("01:02:03.123456")?;
            assert_eq!(naive,
                TryInto::<NaiveTime>::try_into(LocalTime::try_from(naive)?)?);
            Ok(())
        }
    }
}