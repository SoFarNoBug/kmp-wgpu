Pod::Spec.new do |s|
  s.name             = 'KmpWgpuNative'
  s.version          = '0.1.0'
  s.summary          = 'kmp_wgpu_native xcframework (ADR 0015)'
  s.homepage         = 'https://github.com/SoFarNoBug/kmp-wgpu'
  s.license          = { :type => 'MIT' }
  s.author           = { 'jlj' => 'sofarnobug' }
  s.platform         = :ios, '12.0'
  s.vendored_frameworks = 'dist/ios/KmpWgpuNative.xcframework'
  s.source           = { :path => '.' }
  s.requires_arc     = true
end
