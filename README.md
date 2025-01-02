# Bevy Game

Based on: [Bevy Template](https://github.com/TheBevyFlock/bevy_new_2d)

## Run your game

Running your game locally is very simple:

- Use `cargo run` to run a native dev build.
If you're using [VS Code](https://code.visualstudio.com/), this template comes with a [`.vscode/tasks.json`](./.vscode/tasks.json) file.

<details>
  <summary>Run release builds</summary>

- Use `cargo run --profile release-native --no-default-features` to run a native release build.

</details>

<details>
  <summary>Linux dependencies</summary>

If you are using Linux, make sure you take a look at Bevy's [Linux dependencies](https://github.com/bevyengine/bevy/blob/main/docs/linux_dependencies.md).
Note that this template enables Wayland support, which requires additional dependencies as detailed in the link above.
Wayland is activated by using the `bevy/wayland` feature in the [`Cargo.toml`](./Cargo.toml).

</details>

<details>
    <summary>(Optional) Improve your compile times</summary>

[`.cargo/config_fast_builds.toml`](./.cargo/config_fast_builds.toml) contains documentation on how to set up your environment to improve compile times.
After you've fiddled with it, rename it to `.cargo/config.toml` to enable it.

</details>

## Releases

The Game uses [GitHub workflows](https://docs.github.com/en/actions/using-workflows) to run tests and build releases.

## Development
To Activate Developer Tools see dev_tools.rs.

## Known Issues

- You can increase the compilation speed by copying .cargo/config_fast_builds to .cargo/config and adjust it. This requires you to install some additional dependencies to your system.

## License

The source code in this repository is licensed under the following licence:

- [License](./LICENSE.txt)

## Credits

The [assets](./assets) in this repository are all 3rd-party. See the [credits screen](./src/screens/credits.rs) for more information.
