## Contributing

Contributions are what make the open source community such an amazing place to learn, inspire, and create. Any contributions you make are **greatly appreciated**.

### Creating a Pull Request

1. Fork the Project
2. Create your Feature Branch (`git checkout -b feat/new-feature`)
3. Commit your Changes (`git commit -m 'Add some feature'`)
4. Push to the Branch (`git push origin feat/new-feature`)
5. Open a Pull Request

### Testing your changes

Using Docker is the easiest way to to test your code before submitting a pull request. 

> [!NOTE]
> When using the Docker container on Windows, the WSL engine does not support the default collection for keys or tokens. This means that when testing inside the container GitHub tokens will not be stored, even when `komac token update` is used.
> 
> This is a [known issue](https://github.com/hwchen/keyring-rs/blob/47c8daf3e6178a2282ae3e8670d1ea7fa736b8cb/src/secret_service.rs#L73-L77) which is documented in the keyring crate.
>
> As a workaround, you can set the `GITHUB_TOKEN` environment variable from within the container, in the `docker run` command, or in the Dockerfile itself

1. Ensure you have docker installed and the docker engine is running.
2. Run `docker build ./ --tag komac_dev:latest`.
3. Wait for the build to complete.
4. Start the container using `docker run -it komac_dev bash`.
5. Test out any commands. Use the `exit` command to quit the container