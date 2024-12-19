puts "Ruby #{RUBY_VERSION}"
puts 'ASCII whitespace characters for Regexp \s:'
pattern = /^\s$/
(0..255).each do |c|
  s = c.chr(Encoding::ASCII_8BIT)
  if s =~ pattern
    puts "0x#{c.to_s(16).rjust(2, '0')} (#{s.inspect})"
  end
end
