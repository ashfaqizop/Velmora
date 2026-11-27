# Keep classes and constructors looked up from jni_bridge.cpp.
-keep class com.velmora.stepbyte.usbaudio.model.DacFormat { <init>(...); }
-keep class com.velmora.stepbyte.usbaudio.model.DacCapabilities { <init>(...); }

# Keep names used by the exported JNI method symbols.
-keepclasseswithmembernames class com.velmora.stepbyte.usbaudio.UsbAudioDriver { native <methods>; }
