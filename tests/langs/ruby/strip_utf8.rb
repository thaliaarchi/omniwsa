puts "Ruby #{RUBY_VERSION}"
puts "UTF-8 whitespace characters for String#strip:"
(0..0x10FFFF).each do |c|
  next if c.between?(0xD800, 0xDFFF)
  s = c.chr(Encoding::UTF_8)
  both = "#{s}x#{s}"
  stripped = both.strip
  if stripped != both
    direction = if stripped == "x"
      "on both sides"
    elsif stripped == "x#{s}"
      "only on the left"
    elsif stripped == "#{s}x"
      "only on the right"
    else
      "BUG"
    end
    puts "U+#{c.to_s(16).upcase.rjust(4, '0')} (#{s.inspect}) is stripped #{direction}"
  end
end
