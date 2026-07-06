# edmdata

An unofficial tool for aggregating and analyzing Early Day Motion data.

edmdata is primarily used for aggregating statistics about Early Day Motions into JSON files, and buiding SVG/PNG graphs of those statistics.

## Usage

If you would like to create statistics for your own purposes, edmdata can be imported as a library, by adding this to your `Cargo.toml`:
```
edmdata = { git = "https://github.com/k8ey/edmdata.git" }
```
<br>

Before doing anything else, you need to import the database.<br>
Archives of the database can be found in the [releases](https://github.com/k8ey/edmdata/releases) section of this repository.<br>
Download the most recent database archive, extract it into the root of `edmdata`, and then execute the following commands from the root of `edmdata`:<br>
- `mkdir -p data/db`
- `docker run --rm -d -v ./data/db:/data/db -p 27017:27017 mongo`
- `mongorestore --drop --archive=datasets.bak mongodb://127.0.0.1`<br>
> You will want to kill the docker container before running the below commands - or just skip the `docker run` command.
<br>

To run the edm240 example, execute the following set of commands, from the root of `edmdata`:
- `docker run --rm -d -v ./data/db:/data/db -p 27017:27017 mongo`
- `cargo r -r --example edm240`

## Caveats

I'm unlikely to mantain this project, as it is only meant to be a way for others to validate the statistics from https://github.com/k8ey/edm240.

This project is not meant to be production-grade - it was just created as a means of collecting data and statistics about Early Day Motions, in a reasonable time frame.<br>
Because of this, it is missing documentation and comments, and some of the code is rather inefficient.<br>

## License

This repository is dual-licensed under either:
- MIT License ([LICENSE-MIT](LICENSE-MIT) or https://opensource.org/licenses/MIT)
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)

at your option. Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual-licensed as above, without any additional terms or conditions.

This repository also contains Parliamentary information licensed under the [Open Parliament Licence v3.0](https://www.parliament.uk/site-information/copyright-parliament/open-parliament-licence/).
