Pod::Spec.new do |s|
  s.name             = 'KmpWgpuNative'
  s.version          = '0.1.0'
  s.summary          = 'iOS native wgpu runtime (xcframework) for fullscreen GPU present and headless compute'
  s.description      = 'Vendored libkmp_wgpu_native for iOS. Use with Maven io.github.sofarnobug:kmp-wgpu / kmp-wgpu-surface for the Kotlin API.'
  s.homepage         = 'https://github.com/SoFarNoBug/kmp-wgpu'
  s.license          = { :type => 'MIT', :file => 'LICENSE' }
  s.author           = { 'jlj' => '1396275290@qq.com' }
  s.platform         = :ios, '12.0'
  s.source           = { :git => 'https://github.com/SoFarNoBug/kmp-wgpu.git', :tag => s.version.to_s }
  s.vendored_frameworks = 'dist/ios/KmpWgpuNative.xcframework'
  s.requires_arc     = true
end
