import wormhole;

namespace sdf
{

float4 normal(functype(float4)->float f, float4 p)
{
    let helper = float2(0.001, 0.0);
    return normalize(float4(
        f(p + helper.xyyy) - f(p - helper.xyyy),
        f(p + helper.yxyy) - f(p - helper.yxyy),
        f(p + helper.yyxy) - f(p - helper.yyxy),
        f(p + helper.yyyx) - f(p - helper.yyyx)));
}

float torus(float4 p, float ring_radius, float radius)
{
    let position = float4(0.0, 0.0, 0.0, 0.0);
    let q = float2(length((p - position).xyz) - ring_radius, (p - position).w);
    return length(q) - radius;
}

float cut_plane(float2 p, float height, float smooth)
{
    float2 d = float2(p.x + smooth, abs(p.y) - height + smooth);
    return length(max(d, 0)) + min(max(d.x, d.y), 0) - smooth;
}

float wormholes(float4 p, IArray<Wormhole> wormholes, int wormholes_len, float height, float join)
{
    float distance = float.maxValue;
    bool in_wormhole = false;
    for (int i = 0; i < wormholes_len; i++)
    {
        Wormhole wormhole = wormholes[i];
        if (length(p.xyz - wormhole.position) < wormhole.throat_size + wormhole.corner_radius + height)
        {
            distance = min(cut_plane(float2(-length(p.xyz - wormhole.position) + wormhole.throat_size, p.w), height, wormhole.corner_radius), distance);
            in_wormhole = true;
        }
    }
    if (in_wormhole)
    {
        return distance;
    }
    else
    {
        return cut_plane(float2(p.x - join - height, p.w), height, height);
    }
}

}
