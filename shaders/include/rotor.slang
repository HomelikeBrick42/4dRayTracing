struct Rotor
{
    float s;
    float e1e2;
    float e1e3;
    float e1e4;
    float e2e3;
    float e2e4;
    float e3e4;
    float e1e2e3e4;

    Rotor then(Rotor then)
    {
        let _0 = this.s;
        let _1 = this.e1e2;
        let _2 = this.e1e3;
        let _3 = this.e1e4;
        let _4 = this.e2e3;
        let _5 = this.e2e4;
        let _6 = this.e3e4;
        let _7 = this.e1e2e3e4;
        let _8 = then.s;
        let _9 = then.e1e2;
        let _10 = then.e1e3;
        let _11 = then.e1e4;
        let _12 = then.e2e3;
        let _13 = then.e2e4;
        let _14 = then.e3e4;
        let _15 = then.e1e2e3e4;

        var result : Rotor;
        result.s = (_0 * _8) + (_15 * _7) + (-1.0 * _1 * _9) + (-1.0 * _10 * _2) + (-1.0 * _11 * _3) + (-1.0 * _12 * _4) + (-1.0 * _13 * _5) + (-1.0 * _14 * _6);
        result.e1e2 = (_0 * _9) + (_1 * _8) + (_12 * _2) + (_13 * _3) + (-1.0 * _10 * _4) + (-1.0 * _11 * _5) + (-1.0 * _14 * _7) + (-1.0 * _15 * _6);
        result.e1e3 = (_0 * _10) + (_13 * _7) + (_14 * _3) + (_15 * _5) + (_2 * _8) + (_4 * _9) + (-1.0 * _1 * _12) + (-1.0 * _11 * _6);
        result.e1e4 = (_0 * _11) + (_10 * _6) + (_3 * _8) + (_5 * _9) + (-1.0 * _1 * _13) + (-1.0 * _12 * _7) + (-1.0 * _14 * _2) + (-1.0 * _15 * _4);
        result.e2e3 = (_0 * _12) + (_1 * _10) + (_14 * _5) + (_4 * _8) + (-1.0 * _11 * _7) + (-1.0 * _13 * _6) + (-1.0 * _15 * _3) + (-1.0 * _2 * _9);
        result.e2e4 = (_0 * _13) + (_1 * _11) + (_10 * _7) + (_12 * _6) + (_15 * _2) + (_5 * _8) + (-1.0 * _14 * _4) + (-1.0 * _3 * _9);
        result.e3e4 = (_0 * _14) + (_11 * _2) + (_13 * _4) + (_6 * _8) + (-1.0 * _1 * _15) + (-1.0 * _10 * _3) + (-1.0 * _12 * _5) + (-1.0 * _7 * _9);
        result.e1e2e3e4 = (_0 * _15) + (_1 * _14) + (_11 * _4) + (_12 * _3) + (_6 * _9) + (_7 * _8) + (-1.0 * _10 * _5) + (-1.0 * _13 * _2);
        return result;
    }

    Rotor reverse()
    {
        var result : Rotor;
        result.s = this.s;
        result.e1e2 = -this.e1e2;
        result.e1e3 = -this.e1e3;
        result.e1e4 = -this.e1e4;
        result.e2e3 = -this.e2e3;
        result.e2e4 = -this.e2e4;
        result.e3e4 = -this.e3e4;
        result.e1e2e3e4 = this.e1e2e3e4;
        return result;
    }

    float4 rotate_direction(float4 direction)
    {
        let _0 = this.s;
        let _1 = this.e1e2;
        let _2 = this.e1e3;
        let _3 = this.e1e4;
        let _4 = this.e2e3;
        let _5 = this.e2e4;
        let _6 = this.e3e4;
        let _7 = this.e1e2e3e4;
        let _8 = direction.x;
        let _9 = direction.y;
        let _10 = direction.z;
        let _11 = direction.w;

        var result : float4;
        result.x =
            (_8) + (-2.0 * _0 * _1 * _9) + (-2.0 * _0 * _10 * _2) + (-2.0 * _0 * _11 * _3) + (-2.0 * _1 * _1 * _8) + (-2.0 * _10 * _3 * _6) + (-2.0 * _11 * _4 * _7) + (-2.0 * _2 * _2 * _8) + (-2.0 * _2 * _4 * _9) + (-2.0 * _3 * _3 * _8) + (-2.0 * _3 * _5 * _9) + (-2.0 * _6 * _7 * _9) + (-2.0 * _7 * _7 * _8) + (2.0 * _1 * _10 * _4) + (2.0 * _1 * _11 * _5) + (2.0 * _10 * _5 * _7) + (2.0 * _11 * _2 * _6);
        result.y =
            (_9) + (-2.0 * _0 * _10 * _4) + (-2.0 * _0 * _11 * _5) + (-2.0 * _1 * _1 * _9) + (-2.0 * _1 * _10 * _2) + (-2.0 * _1 * _11 * _3) + (-2.0 * _10 * _3 * _7) + (-2.0 * _10 * _5 * _6) + (-2.0 * _2 * _4 * _8) + (-2.0 * _3 * _5 * _8) + (-2.0 * _4 * _4 * _9) + (-2.0 * _5 * _5 * _9) + (-2.0 * _7 * _7 * _9) + (2.0 * _0 * _1 * _8) + (2.0 * _11 * _2 * _7) + (2.0 * _11 * _4 * _6) + (2.0 * _6 * _7 * _8);
        result.z =
            (_10) + (-2.0 * _0 * _11 * _6) + (-2.0 * _1 * _11 * _7) + (-2.0 * _1 * _2 * _9) + (-2.0 * _10 * _2 * _2) + (-2.0 * _10 * _4 * _4) + (-2.0 * _10 * _6 * _6) + (-2.0 * _10 * _7 * _7) + (-2.0 * _11 * _2 * _3) + (-2.0 * _11 * _4 * _5) + (-2.0 * _3 * _6 * _8) + (-2.0 * _5 * _6 * _9) + (-2.0 * _5 * _7 * _8) + (2.0 * _0 * _2 * _8) + (2.0 * _0 * _4 * _9) + (2.0 * _1 * _4 * _8) + (2.0 * _3 * _7 * _9);
        result.w =
            (_11) + (-2.0 * _1 * _3 * _9) + (-2.0 * _10 * _2 * _3) + (-2.0 * _10 * _4 * _5) + (-2.0 * _11 * _3 * _3) + (-2.0 * _11 * _5 * _5) + (-2.0 * _11 * _6 * _6) + (-2.0 * _11 * _7 * _7) + (-2.0 * _2 * _7 * _9) + (2.0 * _0 * _10 * _6) + (2.0 * _0 * _3 * _8) + (2.0 * _0 * _5 * _9) + (2.0 * _1 * _10 * _7) + (2.0 * _1 * _5 * _8) + (2.0 * _2 * _6 * _8) + (2.0 * _4 * _6 * _9) + (2.0 * _4 * _7 * _8);
        return result;
    }

    float4 x()
    {
        let _0 = this.s;
        let _1 = this.e1e2;
        let _2 = this.e1e3;
        let _3 = this.e1e4;
        let _4 = this.e2e3;
        let _5 = this.e2e4;
        let _6 = this.e3e4;
        let _7 = this.e1e2e3e4;

        var result : float4;
        result.x = (1.0) + (-2.0 * _1 * _1) + (-2.0 * _2 * _2) + (-2.0 * _3 * _3) + (-2.0 * _7 * _7);
        result.y = (-2.0 * _2 * _4) + (-2.0 * _3 * _5) + (2.0 * _0 * _1) + (2.0 * _6 * _7);
        result.z = (-2.0 * _3 * _6) + (-2.0 * _5 * _7) + (2.0 * _0 * _2) + (2.0 * _1 * _4);
        result.w = (2.0 * _0 * _3) + (2.0 * _1 * _5) + (2.0 * _2 * _6) + (2.0 * _4 * _7);
        return result;
    }

    float4 y()
    {

        let _0 = this.s;
        let _1 = this.e1e2;
        let _2 = this.e1e3;
        let _3 = this.e1e4;
        let _4 = this.e2e3;
        let _5 = this.e2e4;
        let _6 = this.e3e4;
        let _7 = this.e1e2e3e4;

        var result : float4;
        result.x = (-2.0 * _0 * _1) + (-2.0 * _2 * _4) + (-2.0 * _3 * _5) + (-2.0 * _6 * _7);
        result.y = (1.0) + (-2.0 * _1 * _1) + (-2.0 * _4 * _4) + (-2.0 * _5 * _5) + (-2.0 * _7 * _7);
        result.z = (-2.0 * _1 * _2) + (-2.0 * _5 * _6) + (2.0 * _0 * _4) + (2.0 * _3 * _7);
        result.w = (-2.0 * _1 * _3) + (-2.0 * _2 * _7) + (2.0 * _0 * _5) + (2.0 * _4 * _6);
        return result;
    }

    float4 z()
    {
        let _0 = this.s;
        let _1 = this.e1e2;
        let _2 = this.e1e3;
        let _3 = this.e1e4;
        let _4 = this.e2e3;
        let _5 = this.e2e4;
        let _6 = this.e3e4;
        let _7 = this.e1e2e3e4;

        var result : float4;
        result.x = (-2.0 * _0 * _2) + (-2.0 * _3 * _6) + (2.0 * _1 * _4) + (2.0 * _5 * _7);
        result.y = (-2.0 * _0 * _4) + (-2.0 * _1 * _2) + (-2.0 * _3 * _7) + (-2.0 * _5 * _6);
        result.z = (1.0) + (-2.0 * _2 * _2) + (-2.0 * _4 * _4) + (-2.0 * _6 * _6) + (-2.0 * _7 * _7);
        result.w = (-2.0 * _2 * _3) + (-2.0 * _4 * _5) + (2.0 * _0 * _6) + (2.0 * _1 * _7);
        return result;
    }

    float4 w()
    {
        let _0 = this.s;
        let _1 = this.e1e2;
        let _2 = this.e1e3;
        let _3 = this.e1e4;
        let _4 = this.e2e3;
        let _5 = this.e2e4;
        let _6 = this.e3e4;
        let _7 = this.e1e2e3e4;

        var result : float4;
        result.x = (-2.0 * _0 * _3) + (-2.0 * _4 * _7) + (2.0 * _1 * _5) + (2.0 * _2 * _6);
        result.y = (-2.0 * _0 * _5) + (-2.0 * _1 * _3) + (2.0 * _2 * _7) + (2.0 * _4 * _6);
        result.z = (-2.0 * _0 * _6) + (-2.0 * _1 * _7) + (-2.0 * _2 * _3) + (-2.0 * _4 * _5);
        result.w = (1.0) + (-2.0 * _3 * _3) + (-2.0 * _5 * _5) + (-2.0 * _6 * _6) + (-2.0 * _7 * _7);
        return result;
    }

    RotorMagnitude magnitude_squared()
    {
        let _0 = this.s;
        let _1 = this.e1e2;
        let _2 = this.e1e3;
        let _3 = this.e1e4;
        let _4 = this.e2e3;
        let _5 = this.e2e4;
        let _6 = this.e3e4;
        let _7 = this.e1e2e3e4;

        var result : RotorMagnitude;
        result.s = (_0 * _0) + (_1 * _1) + (_2 * _2) + (_3 * _3) + (_4 * _4) + (_5 * _5) + (_6 * _6) + (_7 * _7);
        result.e1e2e3e4 = (-2.0 * _1 * _6) + (-2.0 * _3 * _4) + (2.0 * _0 * _7) + (2.0 * _2 * _5);
        return result;
    }

    Rotor _normalised(RotorMagnitude inverse_magnitude)
    {
        let _0 = this.s;
        let _1 = this.e1e2;
        let _2 = this.e1e3;
        let _3 = this.e1e4;
        let _4 = this.e2e3;
        let _5 = this.e2e4;
        let _6 = this.e3e4;
        let _7 = this.e1e2e3e4;
        let _8 = inverse_magnitude.s;
        let _9 = inverse_magnitude.e1e2e3e4;

        var result : Rotor;
        result.s = (_0 * _8) + (_7 * _9);
        result.e1e2 = (_1 * _8) + (-1.0 * _6 * _9);
        result.e1e3 = (_2 * _8) + (_5 * _9);
        result.e1e4 = (_3 * _8) + (-1.0 * _4 * _9);
        result.e2e3 = (_4 * _8) + (-1.0 * _3 * _9);
        result.e2e4 = (_2 * _9) + (_5 * _8);
        result.e3e4 = (_6 * _8) + (-1.0 * _1 * _9);
        result.e1e2e3e4 = (_0 * _9) + (_7 * _8);
        return result;
    }

    Rotor normalised()
    {
        let squared_magnitude = this.magnitude_squared();

        let sum = 1.0 / sqrt(squared_magnitude.s + squared_magnitude.e1e2e3e4);
        let prod = squared_magnitude.e1e2e3e4 / (2.0 * (e1e2e3e4 * e1e2e3e4 - squared_magnitude.s * squared_magnitude.s));
        let sqrt_part = sqrt(sum * sum - 4.0 * prod);

        var inverse_magnitude : RotorMagnitude;
        inverse_magnitude.s = (sum + sqrt_part) * 0.5;
        inverse_magnitude.e1e2e3e4 = (sum - sqrt_part) * 0.5;
        return this._normalised(inverse_magnitude);
    }

    static Rotor from_to_vector(float4 from, float4 to)
    {
        let _0 = from.x;
        let _1 = from.y;
        let _2 = from.z;
        let _3 = from.w;
        let _4 = to.x;
        let _5 = to.y;
        let _6 = to.z;
        let _7 = to.w;

        var result : Rotor;
        result.s = (1.0) + (_0 * _4) + (_1 * _5) + (_2 * _6) + (_3 * _7);
        result.e1e2 = (_0 * _5) + (-1.0 * _1 * _4);
        result.e1e3 = (_0 * _6) + (-1.0 * _2 * _4);
        result.e1e4 = (_0 * _7) + (-1.0 * _3 * _4);
        result.e2e3 = (_1 * _6) + (-1.0 * _2 * _5);
        result.e2e4 = (_1 * _7) + (-1.0 * _3 * _5);
        result.e3e4 = (_2 * _7) + (-1.0 * _3 * _6);

        let squared_magnitude = result.magnitude_squared();
        let inverse_magnitude = 1.0 / sqrt(squared_magnitude.s); // optimisation, because this rotor is not a double rotation, the e1e2e3e4 part should be 0
        result.s *= inverse_magnitude;
        result.e1e2 *= inverse_magnitude;
        result.e1e3 *= inverse_magnitude;
        result.e1e4 *= inverse_magnitude;
        result.e2e3 *= inverse_magnitude;
        result.e2e3 *= inverse_magnitude;
        result.e2e4 *= inverse_magnitude;
        result.e3e4 *= inverse_magnitude;

        return result;
    }
}

struct RotorMagnitude
{
    float s;
    float e1e2e3e4;
}
