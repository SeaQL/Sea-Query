use quote::{quote, ToTokens};
use sea_query::*;

// #[test]
// fn with_quote_1() {
//     for (interval_field, token_stream) in [
//         (PgInterval::Year, quote! { PgInterval::Year }),
//         (PgInterval::Month, quote! { PgInterval::Month }),
//         (PgInterval::Day, quote! { PgInterval::Day }),
//         (PgInterval::Hour, quote! { PgInterval::Hour }),
//         (PgInterval::Minute, quote! { PgInterval::Minute }),
//         (PgInterval::Second, quote! { PgInterval::Second }),
//         (PgInterval::YearToMonth, quote! { PgInterval::YearToMonth }),
//         (PgInterval::DayToHour, quote! { PgInterval::DayToHour }),
//         (PgInterval::DayToMinute, quote! { PgInterval::DayToMinute }),
//         (PgInterval::DayToSecond, quote! { PgInterval::DayToSecond }),
//         (
//             PgInterval::HourToMinute,
//             quote! { PgInterval::HourToMinute },
//         ),
//         (
//             PgInterval::HourToSecond,
//             quote! { PgInterval::HourToSecond },
//         ),
//         (
//             PgInterval::MinuteToSecond,
//             quote! { PgInterval::MinuteToSecond },
//         ),
//     ] {
//         assert_eq!(
//             interval_field.into_token_stream().to_string(),
//             token_stream.to_string()
//         );
//     }
// }
