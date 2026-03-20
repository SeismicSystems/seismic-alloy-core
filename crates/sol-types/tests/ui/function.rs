use alloy_sol_types::sol;

sol! {
    function missingParens;
}

sol! {
    function missingSemi1()
}

sol! {
    function missingSemi2() external
}

sol! {
    function missingSemi3() returns (uint256)
}

// OK
sol! {
    function semiNotBrace1() {}
}

// OK
sol! {
    function semiNotBrace2() external {}
}

// OK
sol! {
    function semiNotBrace3() returns (uint256) {}
}

sol! {
    function singleComma(,);
}

// OK
sol! {
    function trailingComma1(bytes,);
    function trailingComma2(bytes a,);
    function trailingComma3(bytes memory a,);
}

sol! {
    function badReturn1() returns;
}

sol! {
    function badReturn2() returns();
}

// OK
sol! {
    function a() private;
    function b() internal;
    function c() public;
    function d() external;

    function e() pure;
    function f() view;
    function g() constant;
    function h() payable;

    function i() virtual;
    function j() immutable;

    function k() override(Interface.k);
    function l() myModifier("a", 0);

    function m() external view returns (uint256);
    function n() public pure returns (uint256,);
}

// Shielded return types are not allowed
sol! {
    function shieldedReturn1() returns (suint256);
}

sol! {
    function shieldedReturn2(saddress to) returns (sbytes32);
}

sol! {
    function shieldedReturn3() returns (sbool);
}

// OK: shielded params with non-shielded returns
sol! {
    function shieldedParams(suint256 amount, saddress to) returns (uint256);
}

fn main() {}
