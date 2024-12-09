import Data.Char (toLower, chr)

main :: IO ()
main = do
    let chars = [chr c | c <- [0x80..0x10FFFF], toLower (chr c) <= '\x7f']
    putStrLn "Characters which Data.Char.toLower maps to ASCII:"
    mapM_ (\ch -> putStrLn (ch : " -> " ++ [toLower ch])) chars
