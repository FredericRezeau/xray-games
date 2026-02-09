#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, xdr::ToXdr, Address, Env};

#[contract]
pub struct MockOhloss;

#[contracttype]
#[derive(Clone, Debug)]
pub struct Player {
    pub selected_faction: u32,
    pub time_multiplier_start: u64,
    pub last_epoch_balance: i128,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct GameSession {
    pub game_id: Address,
    pub player1: Address,
    pub player2: Address,
    pub player1_wager: i128,
    pub player2_wager: i128,
    pub active: bool,
}

#[contracttype]
pub enum DataKey {
    Session(u32),
    PlayerFaction(Address),
}

#[contractimpl]
impl MockOhloss {
    pub fn start_game(
        env: Env,
        game_id: Address,
        session_id: u32,
        player1: Address,
        player2: Address,
        player1_wager: i128,
        player2_wager: i128,
    ) {
        let session = GameSession {
            game_id: game_id.clone(),
            player1: player1.clone(),
            player2: player2.clone(),
            player1_wager,
            player2_wager,
            active: true,
        };
        env.storage().temporary().set(&DataKey::Session(session_id), &session);
    }

    pub fn end_game(env: Env, session_id: u32, player1_won: bool) {
        if let Some(mut session) = env.storage().temporary().get::<_, GameSession>(&DataKey::Session(session_id)) {
            session.active = false;
            env.storage().temporary().set(&DataKey::Session(session_id), &session);
        }
        let _ = player1_won;
    }

    pub fn get_player(env: Env, player: Address) -> Player {
        if let Some(faction) = env.storage().persistent().get::<_, u32>(&DataKey::PlayerFaction(player.clone())) {
            return Player {
                selected_faction: faction,
                time_multiplier_start: 0,
                last_epoch_balance: 1_000_000_000,
            };
        }

        let bytes = player.to_string();
        let mut hash: u64 = 0;
        for byte in bytes.to_xdr(&env).iter() {
            hash = hash.wrapping_mul(31).wrapping_add(byte as u64);
        }
        let faction = (hash % 3) as u32;
        Player {
            selected_faction: faction,
            time_multiplier_start: 0,
            last_epoch_balance: 1_000_000_000,
        }
    }

    pub fn set_player_faction(env: Env, player: Address, faction: u32) {
        if faction > 2 {
            panic!("invalid faction");
        }
        env.storage().persistent().set(&DataKey::PlayerFaction(player), &faction);
    }

    pub fn get_session(env: Env, session_id: u32) -> Option<GameSession> {
        env.storage().temporary().get(&DataKey::Session(session_id))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::testutils::Address as _;

    #[test]
    fn test_full_game_flow() {
        let env = Env::default();
        let contract = env.register(MockOhloss, ());
        let client = MockOhlossClient::new(&env, &contract);
        let game = Address::generate(&env);
        let player1 = Address::generate(&env);
        let player2 = Address::generate(&env);
        client.start_game(&game, &12345, &player1, &player2, &100, &50);
        let session = client.get_session(&12345).unwrap();
        assert!(session.active);
        assert_eq!(session.player1_wager, 100);
        client.end_game(&12345, &true);
        let session = client.get_session(&12345).unwrap();
        assert!(!session.active);
    }

    #[test]
    fn test_player_faction_random() {
        let env = Env::default();
        let contract = env.register(MockOhloss, ());
        let client = MockOhlossClient::new(&env, &contract);
        let mut factions = [0u32; 3];
        for _ in 0..100 {
            let player = Address::generate(&env);
            let data = client.get_player(&player);
            factions[data.selected_faction as usize] += 1;
        }
        let non_zero = factions.iter().filter(|&&x| x > 0).count();
        assert!(non_zero >= 2, "Expected variety in factions");
    }

    #[test]
    fn test_set_player_faction() {
        let env = Env::default();
        let contract = env.register(MockOhloss, ());
        let client = MockOhlossClient::new(&env, &contract);
        let player = Address::generate(&env);
        client.set_player_faction(&player, &2);
        let data = client.get_player(&player);
        assert_eq!(data.selected_faction, 2);
    }

    #[test]
    #[should_panic(expected = "invalid faction")]
    fn test_invalid_faction() {
        let env = Env::default();
        let contract = env.register(MockOhloss, ());
        let client = MockOhlossClient::new(&env, &contract);
        let player = Address::generate(&env);
        client.set_player_faction(&player, &3);
    }
}