```
## LastFm Song Scrobble
Simple (and hardcoded for now) last.fm automatic song scrobbler so you can milk your favorite song as much as you want to.

Right now $uicideboy$ are hardcoded as a song that gets scrobbled but you can change it in `src/main.rs` on line 34 and 35.

## Setup
 1. Rename `.env.example` file to `.env`
 2. Create last.fm application (https://www.last.fm/api/account/create)
 3. Fill `.env` with your app details
 4. Compile and run it lol?

## Limitations
As far as I know last.fm has limit of 2800 scrobbles per day per account
```