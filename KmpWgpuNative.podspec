Pod::Spec.new do |s|
  s.name             = 'KmpWgpuNative'
  s.version          = '0.1.0'
  s.summary          = 'iOS native wgpu runtime (xcframework) for fullscreen GPU present and headless compute'
  s.description      = 'Vendored libkmp_wgpu_native for iOS. Use with Maven io.github.sofarnobug:kmp-wgpu / kmp-wgpu-surface for the Kotlin API.'
  s.homepage         = 'https://github.com/SoFarNoBug/kmp-wgpu'
  s.license          = { :type => 'MIT', :file => 'LICENSE' }
  s.author           = { 'jlj' => '1396275290@qq.com' }
  # Xcode 16+/27 SDK：模拟器最低部署目标 15.0
  s.platform         = :ios, '15.0'
  s.source           = { :git => 'https://github.com/SoFarNoBug/kmp-wgpu.git', :tag => s.version.to_s }
  # 路径相对仓库根（本 podspec 放在仓根）
  s.vendored_frameworks = 'kmp-wgpu-native/dist/ios/KmpWgpuNative.xcframework'
  s.requires_arc     = true
  # 当前 xcframework 仅 arm64（含 sim）；排除 Intel 模拟器切片以免 lint/链接失败
  s.pod_target_xcconfig = { 'EXCLUDED_ARCHS[sdk=iphonesimulator*]' => 'x86_64' }
  s.user_target_xcconfig = { 'EXCLUDED_ARCHS[sdk=iphonesimulator*]' => 'x86_64' }
end
