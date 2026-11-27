# JNA looks classes and fields up by name, and UniFFI's generated bindings
# are called through it.
-keep class com.sun.jna.** { *; }
-keepclassmembers class * extends com.sun.jna.** { public *; }
-dontwarn java.awt.**
-keep class com.velmora.stepbyte.engine.ffi.** { *; }
