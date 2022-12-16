# Rust Analytics for Snowplow

## Documentation

Documentation for the snowplow rust tracker can be viewed by running `cargo doc --open`

## Architecture

At this time, the snowplow tracker is closed source. It is hosted here on github in anticipation that it will be open-sourced as a fork of the official snowplow tracker once we're ready to announce the telemetry initiative to the general public.

In the meantime, to ease depending on this library, we've mirrored it to our private gitlab: https://gitlab.1password.io/dev/core/snowplow-rust-tracker-mirror. That gitlab repo is an autonomous mirror that will automatically grab changes to this github repo and it's designed to be depended on directly (though also see the README in `foundation/op-snowplow/`).

Issues related to bringing this repo online are generally discoverable from https://gitlab.1password.io/dev/core/core/-/issues/18229.

## Open source checklist

- [ ] Correctly license as Apache 2.0
  - [ ] Include a NOTICE file
- [ ] Replace this README with real docs
- [ ] Rename the crate so we don't conflict with the official tracker

## Copyright and License

This Snowplow Rust Tracker is copyright 2022 1Password.

Licensed under the **[Apache License, Version 2.0][license]** (the "License"); you may not use this software except in compliance with the License.

Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.

This Snowplow Rust Tracker is originally copyright 2022 Snowplow Analytics Ltd.

[website]: https://snowplow.io
[snowplow]: https://github.com/snowplow/snowplow
[docs]: https://docs.snowplow.io/
[rust-docs]: https://docs.snowplow.io/docs/collecting-data/collecting-from-own-applications/rust-tracker/
[gh-actions]: https://github.com/snowplow-incubator/snowplow-rust-tracker/actions/workflows/build.yml
[gh-actions-image]: https://github.com/snowplow-incubator/snowplow-rust-tracker/actions/workflows/build.yml/badge.svg
[license]: https://www.apache.org/licenses/LICENSE-2.0
[license-image]: https://img.shields.io/badge/license-Apache--2-blue.svg?style=flat
[releases]: https://crates.io/crates/snowplow_tracker
[techdocs]: https://docs.snowplow.io/docs/collecting-data/collecting-from-own-applications/rust-tracker/
[techdocs-image]: https://d3i6fms1cm1j0i.cloudfront.net/github/images/techdocs.png
[setup]: https://docs.snowplow.io/docs/collecting-data/collecting-from-own-applications/rust-tracker/quick-start-guide
[setup-image]: https://d3i6fms1cm1j0i.cloudfront.net/github/images/setup.png
[api-docs]: https://snowplow.github.io/snowplow-rust-tracker/
[contributing-image]: https://d3i6fms1cm1j0i.cloudfront.net/github/images/contributing.png
[tracker-classificiation]: https://github.com/snowplow/snowplow/wiki/Tracker-Maintenance-Classification
[early-release]: https://img.shields.io/static/v1?style=flat&label=Snowplow&message=Early%20Release&color=014477&labelColor=9ba0aa&logo=data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABAAAAAQCAMAAAAoLQ9TAAAAeFBMVEVMaXGXANeYANeXANZbAJmXANeUANSQAM+XANeMAMpaAJhZAJeZANiXANaXANaOAM2WANVnAKWXANZ9ALtmAKVaAJmXANZaAJlXAJZdAJxaAJlZAJdbAJlbAJmQAM+UANKZANhhAJ+EAL+BAL9oAKZnAKVjAKF1ALNBd8J1AAAAKHRSTlMAa1hWXyteBTQJIEwRgUh2JjJon21wcBgNfmc+JlOBQjwezWF2l5dXzkW3/wAAAHpJREFUeNokhQOCA1EAxTL85hi7dXv/E5YPCYBq5DeN4pcqV1XbtW/xTVMIMAZE0cBHEaZhBmIQwCFofeprPUHqjmD/+7peztd62dWQRkvrQayXkn01f/gWp2CrxfjY7rcZ5V7DEMDQgmEozFpZqLUYDsNwOqbnMLwPAJEwCopZxKttAAAAAElFTkSuQmCC
