puts "Ruby #{RUBY_VERSION}"
puts 'UTF-8 whitespace characters for Regexp \s:'
pattern = /^\s$/
(0..0x10FFFF).each do |c|
  next if c.between?(0xD800, 0xDFFF)
  s = c.chr(Encoding::UTF_8)
  if s =~ pattern
    puts "U+#{c.to_s(16).upcase.rjust(4, '0')} (#{s.inspect})"
  end
end
