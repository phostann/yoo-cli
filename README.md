If you wan't to use yoo cli to help you in your work, you should do the following steps:

1. download the yoo.exe file from the release
2. config the env variable

```zsh
# yoo-cli config
export YOO_SERVER=****
export YOO_SERVER_USERNAME=****
export YOO_SERVER_PASSWORD=****
export YOO_GITLAB_SERVER=****
export YOO_GITLAB_USERNAME=****
export YOO_GITLAB_PASSWORD=****
export YOO_GITLAB_TOKEN=****
export YOO_GITLAB_NAMESPACE_ID=****
```

3. link the yoo.exe to your path

```zsh
export PATH=$PATH:/path/to/yoo.exe # replace the path/to/yoo.exe with your yoo.exe path
```

there is some different between the windows and unix, so you should config the env variable in different way.