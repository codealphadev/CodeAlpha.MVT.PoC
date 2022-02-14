# Learnings

An unstructured collection of resources that very helpful.
## Error fixes

* **Problem:** When trying to build a swift project from the command line using `swift build`, this error appears: `xcrun: error: unable to find utility "xctest", not a developer tool or in PATH`.
  
  **Fix:** https://stackoverflow.com/questions/61501298/xcrun-error-unable-to-find-utility-xctest-not-a-developer-tool-or-in-path

## AuthZ/AuthN
* https://developer.apple.com/library/archive/samplecode/EvenBetterAuthorizationSample/Introduction/Intro.html


## Security

* Thread Vector XPC/Mach: https://thecyberwire.com/events/docs/IanBeer_JSS_Slides.pdf 


## Random collection (uncleaned):
* Find out bundle id of an app installed on macOS:
  ```bash
  # Get bundle id of XCode ('com.apple.dt.Xcode')
  /usr/libexec/PlistBuddy -c 'Print CFBundleIdentifier' /Applications/Xcode-beta.app/Contents/Info.plist
  ```
* Accessing text value from any System wide Application via Accessibility API: https://macdevelopers.wordpress.com/2014/01/31/accessing-text-value-from-any-system-wide-application-via-accessibility-api/comment-page-1/



## XPC Stuff

* Anonymous Listeners: https://developer.apple.com/forums/thread/126437
* Darwin NotificationCenter: https://gist.github.com/AvdLee/07de0b0fe7dbc351541ab817b9eb6c1c

