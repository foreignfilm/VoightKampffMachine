This is a experimental fan project, based on and incoporating the assets from [*Inhuman Conditions*](https://robots.management). The exact nature of this project is yet to be determined.

*Inhuman Conditions* is a Kickstarter board game project by Tommy Maranges, Cory O'Brien, Mackenzie Schubert. This repo is not affiliated with Inhuman Conditions creators.

---

To build:

* elm 0.18 (through homebrew, __an old version__)
* rust 1.29+ (through rustup)
* ruby

```
./models.rb
cd client
./build.sh
cd ../server
echo 'RUST_LOG=warp=trace,server=trace' > .env
cargo run
```
