use bitflags::bitflags;
use std::collections::VecDeque;
type PiecePosition = u64;

fn bit_to_position(bit: PiecePosition) -> Result<String, String> {
    if bit == 0 {
        return Err("No piece present!".to_string());
    } else {
        let onebit_index = bit_scan(bit);
        return Ok(index_to_position(onebit_index));
    }
}

fn position_to_bit(position: &str) -> Result<PiecePosition, String> {
    if position.len() != 2 {
        return Err(format!("Invalid length: {}, string: '{}'", position.len(), position));
    }

    let bytes = position.as_bytes();
    let byte0 = bytes[0];
    if byte0 < 97 || byte0 >= 97 + 8 {
        return Err(format!("Invalid column character: {}, string: '{}'", byte0 as char, position));
    }

    let column = (byte0 - 97) as u32;

    let byte1 = bytes[1];
    let row;

    match (byte1 as char).to_digit(10) {
        Some(number) => if number < 1 || number > 8 {
            return Err(format!("Invalid row character: {}, string: '{}'", byte1 as char, position));
        } else {
            row = number - 1;
        },
        None => return Err(format!("Invalid row character: {}, string: '{}'", byte1 as char, position))
    }

    let square_number = row * 8 + column;
    let bit = (1 as u64) << square_number;

    Ok(bit)
}

static COL_MAP: [char; 8] = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];

fn index_to_position(index: usize) -> String {
    let column = index % 8;
    let row = index / 8 + 1;
    return format!("{}{}", COL_MAP[column], row);
}

static MOD67TABLE: [usize; 67] = [
    64, 0, 1, 39, 2, 15, 40, 23, 3, 12, 16, 59, 41, 19, 24, 54, 4, 64, 13, 10, 17, 62, 60, 28, 42,
    30, 20, 51, 25, 44, 55, 47, 5, 32, 64, 38, 14, 22, 11, 58, 18, 53, 63, 9, 61, 27, 29, 50, 43,
    46, 31, 37, 21, 57, 52, 8, 26, 49, 45, 36, 56, 7, 48, 35, 6, 34, 33,
];

fn bit_scan(bit: u64) -> usize {
    let remainder = (bit % 67) as usize;
    return MOD67TABLE[remainder];
}

#[derive(Debug, PartialEq, Clone, Copy, Default)]
enum Color {
    #[default]
    White,
    Black,
}

#[derive(Debug, PartialEq)]
enum PieceType {
    Pawn,
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
}

#[derive(Debug, PartialEq)]
struct Piece {
    position: PiecePosition,
    color: Color,
    piece_type: PieceType,
}

#[derive(Debug, PartialEq)]
enum Square {
    Empty,
    Occupied(usize),
}

bitflags! {
    #[derive(Default)]
    struct CastlingRights: u8 {
        const None = 0;
        const WhiteKingSide = 1 << 0;
        const WhiteQueenSide = 1 << 1;
        const BlackKingSide = 1 << 2;
        const BlackQueenSide = 1 << 3;
        const All = 
            Self::WhiteKingSide.bits()
            | Self::WhiteQueenSide.bits()
            | Self::BlackKingSide.bits()
            | Self::BlackQueenSide.bits();
    }
}


#[derive(Default)]
struct Game {
    pieces: Vec<Piece>,
    squares: Vec<Square>,
    active_color: Color,
    castling_rights: CastlingRights,
    en_passant: Option<PiecePosition>,
    halfmove_clock: usize,
    fullmove_number: usize,
}

impl Game {

    fn init() -> Game {
        let mut game = Game::default(); 

        let mut piece_index = 0;
        let color = Color::White;

        game.push_piece_and_square(0, color, PieceType::Rook, &mut piece_index);
        game.push_piece_and_square(1, color, PieceType::Knight, &mut piece_index);
        game.push_piece_and_square(2, color, PieceType::Bishop, &mut piece_index);
        game.push_piece_and_square(3, color, PieceType::Queen, &mut piece_index);
        game.push_piece_and_square(4, color, PieceType::King, &mut piece_index);
        game.push_piece_and_square(5, color, PieceType::Bishop, &mut piece_index);
        game.push_piece_and_square(6, color, PieceType::Knight, &mut piece_index);
        game.push_piece_and_square(7, color, PieceType::Rook, &mut piece_index);
        
        for i in 8..16 {
            game.push_piece_and_square(i, color, PieceType::Pawn, &mut piece_index);
        }

        for i in 16..48 {
            game.push_empty_square();
        }

        let color = Color::Black;

        for i in 48..56 {
            game.push_piece_and_square(i, color, PieceType::Pawn, &mut piece_index);
        }

        let offset = 56;
        game.push_piece_and_square(0 + offset, color, PieceType::Rook, &mut piece_index);
        game.push_piece_and_square(1 + offset, color, PieceType::Knight, &mut piece_index);
        game.push_piece_and_square(2 + offset, color, PieceType::Bishop, &mut piece_index);
        game.push_piece_and_square(3 + offset, color, PieceType::Queen, &mut piece_index);
        game.push_piece_and_square(4 + offset, color, PieceType::King, &mut piece_index);
        game.push_piece_and_square(5 + offset, color, PieceType::Bishop, &mut piece_index);
        game.push_piece_and_square(6 + offset, color, PieceType::Knight, &mut piece_index);
        game.push_piece_and_square(7 + offset, color, PieceType::Rook, &mut piece_index);
        
        game
    }

    fn default() -> Game {
        Game {
            pieces: vec![],
            squares: vec![],
            active_color: Color::White,
            castling_rights: CastlingRights::All,
            en_passant: None,
            halfmove_clock: 0,
            fullmove_number: 1
        }
    }

    fn to_string(&self) -> String {
        let mut board = "".to_owned();
        let mut temp = "".to_owned();

        for (i, square) in self.squares.iter().enumerate() {
            match square {
                Square::Empty => temp.push_str(&index_to_position(i)),
                Square::Occupied(idx) => temp.push_str(&self.pieces[*idx].to_string())
            }

            if (i + 1) % 8 == 0 {
                temp.push_str("\n");
                board.insert_str(0, &temp);
                temp.clear();
            }
        }
        board.insert_str(0, &temp);
        board
    }

    fn push_piece_and_square(&mut self, position: usize, color: Color, piece_type: PieceType, index: &mut usize) {
        self.pieces.push(Piece {
            position: (1 as u64) << position,
            color: color,
            piece_type: piece_type,
        });

        self.squares.push(Square::Occupied(*index));
        *index += 1;
    }

    fn push_empty_square(&mut self) {
        self.squares.push(Square::Empty);
    }

    fn from_fen(fen: &str) -> Game {
        let mut game = Game::default();

        let (position, rest) = split_on(fen, ' ');

        let mut square_deque = VecDeque::new();
        let mut piece_index = 0;
        let mut piece_position = 64;

        for row in position.splitn(8, |ch| ch == '/') {
            piece_position -= 8;
            let (pieces, squares) = parse_row(&row, piece_index, piece_position);

            for p in pieces {
                game.pieces.push(p);
                piece_index += 1;
            }

            for s in squares {
                square_deque.push_front(s);
            }
        }
        game.squares = Vec::from(square_deque);

        let (color_to_move, rest) = split_on(rest, ' ');
        game.active_color = match color_to_move {
            "w" => Color::White,
            "b" => Color::Black,
            _ => panic!("Unknown color designator: '{}'", color_to_move)
        };

        let (castling_rights, rest) = split_on(rest, ' ');
        let mut castling = CastlingRights::None;
        for ch in castling_rights.chars() {
            match ch {
                'K' => castling |= CastlingRights::WhiteKingSide,
                'Q' => castling |= CastlingRights::WhiteQueenSide,
                'k' => castling |= CastlingRights::BlackKingSide,
                'q' => castling |= CastlingRights::BlackQueenSide,
                '-' => (),
                _ => panic!("Unknown castling rights designator: '{}'", ch)
            }
        }
        game.castling_rights = castling;

        let (en_passent, rest) = split_on(rest, ' ');
        match en_passent {
            "-" => game.en_passant = None,
            s => match position_to_bit(s) {
                Err(msg) => panic!("{}", msg),
                Ok(bit) => game.en_passant = Some(bit)
            }
        };

        let (halfmove_clock, rest) = split_on(rest, ' ');
        match halfmove_clock.parse() {
            Ok(number) => game.halfmove_clock = number,
            Err(_) => panic!("Invalid halfmove: {}", halfmove_clock)
        }

        let (fullmove_number, rest) = split_on(rest, ' ');
        match fullmove_number.parse() {
            Ok(number) => game.fullmove_number = number,
            Err(_) => panic!("Invalid fullmove: {}", fullmove_number)
        }

        game
    }
}

fn parse_row(row: &str, mut piece_index: usize, mut piece_position: usize) -> (Vec<Piece>, VecDeque<Square>) {
    let mut pieces = Vec::new();
    let mut squares = VecDeque::new();

    let mut color;

    macro_rules! add_piece {
        ($piece_type:ident) => {
            {
                let piece = Piece {
                    color: color,
                    position: (1 as u64) << piece_position,
                    piece_type: PieceType::$piece_type
                };
        
                let square = Square::Occupied(piece_index);
                pieces.push(piece);
                squares.push_front(square);
                piece_position += 1;
                piece_index += 1;
            }
        };
    }

    for ch in row.chars() {
        let is_upper = ch.is_ascii_uppercase();
        color = if is_upper { Color::White } else { Color::Black };
        match ch.to_ascii_lowercase() {
            'r' => add_piece!(Rook),
            'n' => add_piece!(Knight),
            'b' => add_piece!(Bishop),
            'q' => add_piece!(Queen),
            'k' => add_piece!(King),
            'p' => add_piece!(Pawn),
            num => {
                match num.to_digit(10) {
                    None => panic!("Invalid input: {}", num),
                    Some(number) => for _ in 0..number {
                        squares.push_front(Square::Empty);
                        piece_position += 1;
                    }
                }
            }
            
        }
    }

    (pieces, squares)
}

fn split_on(s: &str, sep: char) -> (&str, &str) {
    for (i, item) in s.chars().enumerate() {
        if item == sep {
            return (&s[0..i], &s[i+1..]);
        }
    }

    (&s[..], "")
}

impl Piece {
    fn to_string(&self) -> String {
        let mut result = match self.piece_type {
            PieceType::Pawn => "p ",
            PieceType::Rook => "r ",
            PieceType::Knight => "n ",
            PieceType::Bishop => "b ",
            PieceType::Queen => "q ",
            PieceType::King => "k "
        }.to_string();

        if self.color == Color::White {
            result.make_ascii_uppercase();
        }

        result
    }
}

fn main() {
    // let game = Game::init();
    let fen_str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    let game = Game::from_fen(fen_str);

    // let (first_row, rest) = split_on(fen_str, '/');
    // println!("{}\n{}", first_row, rest);
    println!("{}", game.to_string());
    println!("{:?}, {:?}, {}", game.active_color, game.en_passant, game.fullmove_number);
}
