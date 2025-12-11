# ARKAN

This is a React Native Android application designed for ARCAN. The app is currently built only for Android. Right now, it works only when running local server (temporary workaround), receives and displays coordinates from USB receiver. It's designed to work as an offline app but device must has Internet connection at the first launch to download the map (disabled for now).  

## Features

- Shows the map using MapBox  
- Shows the user's coordinates if GPS is enabled  
- Shows ARCAN's coordinates  
- Bug fixes in (version-history/)  

## Prerequisites
- Node.js  
- npm  
- Android Studio  
- Physical Android device or Android emulator  

## Quick Start

Clone the repository, install dependencies, and run the app on Android:  

```bash  
git clone https://github.com/rasthpop/arkan_POC.git  
cd arkan_POC  
git checkout mobile-app  
python3 usb_server.py

npm install  

npx react-native run-android  

#OR to build android app  

cd android  
./gradlew assembleRelease  

#The generated APK will be located at android/app/build/outputs/apk/release/app-release.apk  
```