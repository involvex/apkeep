To download directly from the Google Play Store, you have a few authentication options:

## Option 1: Browser login (recommended for personal accounts)

Run apkeep with `--browser-login`; it opens a browser window at the Google
[embedded setup page](https://accounts.google.com/EmbeddedSetup). Log in to the
Google account you want to use (and accept the Terms of Service if prompted).
apkeep reads the one-time `oauth_token` cookie from that browser session and
exchanges it for a long-lived AAS token, which it prints:

```shell
apkeep --browser-login -e 'someone@gmail.com'
```

By default apkeep tries to find Chrome. You can also select `edge` or `chromium`:

```shell
apkeep --browser-login=edge -e 'someone@gmail.com'
```

Use `--login-timeout` to change how long apkeep waits for the login (default:
300 seconds). Add `--save-token` to write the email and AAS token to the
`apkeep.ini` config file so you do not have to log in again:

```shell
apkeep --browser-login -e 'someone@gmail.com' --save-token
```

Note that the saved token is stored as plain text, so protect the config file
accordingly. The browser uses a temporary profile which is deleted afterwards;
your normal browser session is not touched. This option requires a desktop
session — on a headless machine, use option 2 or 3 below.

An AAS token can then be used to download an app:

```shell
apkeep -a com.instagram.android -e 'someone@gmail.com' -t some_aas_token .
```

## Option 2: Import cookies from a cookies.txt file

If you prefer to collect cookies yourself (for example with a browser
extension), export them in Netscape `cookies.txt` format after logging in at
the [embedded setup page](https://accounts.google.com/EmbeddedSetup). apkeep
finds the `oauth_token` cookie in that file and exchanges it for an AAS token:

```shell
apkeep --cookies /path/to/cookies.txt -e 'someone@gmail.com'
```

`--save-token` works here too. This path does not require a browser window.

## Option 3: Manual OAuth token copy

You can also obtain an OAuth token manually by visiting the Google
[embedded setup page](https://accounts.google.com/EmbeddedSetup) and:

- Open the browser debugging console to the `Network` tab
- Log in
- If the "Google Terms of Services" pop up, click `I agree` (there may be some delay afterwards)
- Select the last request from `accounts.google.com` in the `Network` tab
- Select the `Cookies` tab of this request
- One of the response cookies is `oauth_token`
- Copy the `value` field (it will start with `oauth2_4/`)

It can only be used once, in order to obtain the AAS token which can be used subsequently. To obtain this token:

```shell
apkeep -e 'someone@gmail.com' --oauth-token oauth2_4/...
```

## Option 4: Using AUTH Token (for token dispensers)

If you have an AUTH token from a token dispenser (e.g., Aurora Store's dispenser), you can use it directly with the `--auth-token` flag:

```shell
apkeep -a com.instagram.android -d google-play -e 'dispenser_email@gmail.com' --auth-token 'ya29.a0...' --accept-tos .
```

AUTH tokens typically start with `ya29.`. The `--accept-tos` flag is recommended for first-time use with a new account to automatically accept Google Play's Terms of Service.

## Device Configuration

Both options will use a default device configuration of `px_9a`, a timezone of `UTC`, and a locale of `en_US`. To specify a different device profile, use the `-o` option:

```shell
apkeep -a com.instagram.android -d google-play -o device=ad_g3_pro -e 'someone@gmail.com' -t some_aas_token .
```

Available devices are specified [here](https://github.com/EFForg/rs-google-play/blob/master/gpapi/device.properties).

To specify your own device properties file (e.g. one you've exported from Aurora Store's spoof manager):

```shell
apkeep -a com.instagram.android -d google-play -o device=default,device_properties_file=/path/to/device.properties .
```

Likewise, a separate timezone or locale can also be specified:

```shell
apkeep -a com.instagram.android -d google-play -o device=ad_g3_pro,locale=es_MX -e 'someone@gmail.com' -t some_aas_token .
```

Note that if the app you're downloading is region-locked and not available in your region, the locale and timezone changes alone will not let apkeep download it. 
In that case the download will fail with a `Invalid app response` error. 
To download a region-locked app you first need to bind it to your account by downloading it with an IP address from a region it's available in.
After the first download, you can download the app from any region.


This option attempts to download a split APK if available, and falls back to the full APK:

```shell
apkeep -a hk.easyvan.app.client -d google-play -o split_apk=true -e 'someone@gmail.com' -t some_aas_token .
```

A full list of options:

* `device`: specify a device profile as described above
* `device_properties_file`: specify a custom device properties file
* `locale`: specify a locale
* `timezone`: specify a timezone 
* `split_apk`: when set to `1` or `true`, attempts to download a [split APK](https://developer.android.com/studio/build/configure-apk-splits)
* `include_dex_metadata`: when set to `1` or `true`, attempts to download the DexMetadata (dm) file for an app, which contains the apps' cloud profile
* `include_additional_files`: when set to `1` or `true`, attempts to download any [additional `obb` expansion files](https://developer.android.com/google/play/expansion-files) for the app

If you prefer not to provide your credentials on the command line, you can specify them in a config file named `apkeep.ini`. This config file may have to be created, and must be located in the user config directory under the subpath `apkeep`. Usually on Linux systems this will be `~/.config/apkeep/apkeep.ini`. In this file specify your email and token:

```ini
[google]
email = someone@gmail.com
aas_token = some_aas_token
# OR use auth_token instead of aas_token:
# auth_token = ya29.a0...
```

Optionally, the path to this `ini` file can be specified:

```shell
apkeep -a com.instagram.android -d google-play -i ~/path/to/some.ini .
```
