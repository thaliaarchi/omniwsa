import Data.Char (toLower, chr, ord)
import Text.Printf (printf)

main :: IO ()
main = do
    let chars = [chr c | c <- [0x80..0x10FFFF], toLower (chr c) <= '\x7f']
    putStrLn "Characters which Data.Char.toLower maps to ASCII:"
    mapM_ (\c -> printf "'%c' U+%04X -> '%c' U+%04X\n" c (ord c) (toLower c) (ord (toLower c))) chars
