#[starknet::contract]
mod UsdcMock {
    use openzeppelin::token::erc20::ERC20Component;
    use openzeppelin::token::erc20::interface::IERC20Metadata;
    use openzeppelin::access::ownable::OwnableComponent;
    use starknet::ContractAddress;
    use starknet::storage::Map;
    use core::integer::u256;
    use core::num::traits::OverflowingAdd;

    component!(path: ERC20Component, storage: erc20, event: ERC20Event);
    component!(path: OwnableComponent, storage: ownable, event: OwnableEvent);

    #[abi(embed_v0)]
    impl ERC20Impl = ERC20Component::ERC20Impl<ContractState>;
    #[abi(embed_v0)]
    impl ERC20CamelOnlyImpl = ERC20Component::ERC20CamelOnlyImpl<ContractState>;
    #[abi(embed_v0)]
    impl OwnableImpl = OwnableComponent::OwnableImpl<ContractState>;

    impl ERC20HooksImpl of ERC20Component::ERC20HooksTrait<ContractState> {
        fn before_update(
            ref self: ERC20Component::ComponentState<ContractState>,
            from: ContractAddress,
            recipient: ContractAddress,
            amount: u256,
        ) {}
        fn after_update(
            ref self: ERC20Component::ComponentState<ContractState>,
            from: ContractAddress,
            recipient: ContractAddress,
            amount: u256,
        ) {}
    }

    impl ERC20InternalImpl = ERC20Component::InternalImpl<ContractState>;
    impl OwnableInternalImpl = OwnableComponent::InternalImpl<ContractState>;

    #[storage]
    struct Storage {
        #[substorage(v0)]
        erc20: ERC20Component::Storage,
        #[substorage(v0)]
        ownable: OwnableComponent::Storage,
        last_mint_day: Map::<ContractAddress, u64>,
        minted_today: Map::<ContractAddress, u256>,
    }

    #[event]
    #[derive(Drop, starknet::Event)]
    enum Event {
        #[flat]
        ERC20Event: ERC20Component::Event,
        #[flat]
        OwnableEvent: OwnableComponent::Event,
    }

    #[constructor]
    fn constructor(ref self: ContractState, owner: ContractAddress) {
        self.ownable.initializer(owner);
        self.erc20.initializer("ZapCode USDC Mock", "ZUSDC");
    }

    #[abi(embed_v0)]
    impl ERC20MetadataImpl of IERC20Metadata<ContractState> {
        fn decimals(self: @ContractState) -> u8 {
            6_u8
        }
        fn name(self: @ContractState) -> ByteArray {
            "ZapCode USDC Mock"
        }
        fn symbol(self: @ContractState) -> ByteArray {
            "ZUSDC"
        }
    }

    #[generate_trait]
    #[abi(per_item)]
    impl ExternalImpl of ExternalTrait {
        #[external(v0)]
        fn mint(ref self: ContractState, recipient: ContractAddress, amount: u256) {
            let caller = starknet::get_caller_address();
            let owner = self.ownable.owner();
            if caller == owner {
                self.erc20.mint(recipient, amount);
                return ();
            }

            // 10 USDC at 6 decimals
            let max_daily = u256 { low: 10000000_u128, high: 0_u128 };

            let block_timestamp = starknet::get_block_timestamp();
            let day = block_timestamp / 86400_u64;

            let last_day = self.last_mint_day.read(caller);
            let mut minted = self.minted_today.read(caller);
            if last_day != day {
                minted = u256 { low: 0, high: 0 };
            }

            let (new_total, overflow) = OverflowingAdd::overflowing_add(minted, amount);
            assert(!overflow, 'Overflow in mint calculation');
            assert(new_total <= max_daily, 'Mint limit exceeded for today');

            self.last_mint_day.write(caller, day);
            self.minted_today.write(caller, new_total);

            self.erc20.mint(recipient, amount);
        }
    }
}