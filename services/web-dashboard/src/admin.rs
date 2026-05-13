use axum::{
    extract::{Form, Query, State},
    http::{HeaderMap, StatusCode},
    response::Html,
};
use serde::Deserialize;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::bot_messaging::*;
use crate::AppState;

#[derive(Deserialize)]
pub struct SetEconomyForm {
    pub user_id: String,
    pub username: String,
    pub wallet: i64,
    pub title: String,
    pub pass: String,
}

#[derive(Deserialize)]
pub struct SetRpgForm {
    pub user_id: String,
    pub username: String,
    pub level: i64,
    pub class: String,
    pub exp: i64,
    pub pass: String,
}

#[derive(Deserialize)]
pub struct BroadcastForm {
    pub bot_name: String,
    pub channel_id: String,
    pub message: String,
    pub pass: String,
}

#[derive(Deserialize)]
pub struct AnnounceForm {
    pub bot_name: String,
    pub channel_id: String,
    pub message: String,
    pub pass: String,
}

fn admin_styles() -> &'static str {
    "<style>*{font-family:'Noto Sans Thai',sans-serif}.font-display{font-family:'Orbitron',sans-serif}body{background:#0a0a0f}.bg-grid{background-image:linear-gradient(rgba(168,85,247,0.03) 1px,transparent 1px),linear-gradient(90deg,rgba(168,85,247,0.03) 1px,transparent 1px);background-size:60px 60px}.glow-text{text-shadow:0 0 40px rgba(168,85,247,0.5),0 0 80px rgba(168,85,247,0.2)}.card-glass{background:linear-gradient(135deg,rgba(15,15,25,0.8),rgba(10,10,18,0.9));border:1px solid rgba(255,255,255,0.05);backdrop-filter:blur(20px)}.pulse-dot{animation:pulse 2s infinite}@keyframes pulse{0%,100%{opacity:1}50%{opacity:0.4}}.input-glow:focus{box-shadow:0 0 20px rgba(168,85,247,0.2),inset 0 0 20px rgba(168,85,247,0.03)}.select-custom{appearance:none;background-image:url(\"data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 12 12'%3E%3Cpath fill='%236b7280' d='M6 8L1 3h10z'/%3E%3C/svg%3E\");background-repeat:no-repeat;background-position:right 16px center;padding-right:40px}.tab-btn.active{background:rgba(168,85,247,0.15);color:#c084fc;border-color:rgba(168,85,247,0.3)}.section{display:none}.section.active{display:block}</style>"
}

fn verify_admin(pass: &str) -> bool {
    let admin_pass = std::env::var("ADMIN_PASS").unwrap_or_default();
    pass == admin_pass
}

pub async fn admin_handler(State(app): State<Arc<AppState>>, _headers: HeaderMap, Query(params): Query<HashMap<String, String>>) -> Html<String> {
    let admin_pass = std::env::var("ADMIN_PASS").expect("ADMIN_PASS not set in .env");
    if params.get("pass").is_none() { return admin_login_page(); }
    if params.get("pass") != Some(&admin_pass) {
        return Html(r#"<!DOCTYPE html><html><head><script src="https://cdn.tailwindcss.com"></script></head><body class="bg-[#0a0a0f] min-h-screen flex items-center justify-center"><div class="text-center"><div class="text-8xl mb-6">🚫</div><h1 class="text-3xl font-bold text-red-500 mb-2">ACCESS DENIED</h1><p class="text-gray-500 mb-8">มึงไม่ใช่แอดมิน!</p><a href="https://admin.themostpoordev.top" class="px-6 py-3 rounded-xl bg-red-500/10 border border-red-500/20 text-red-400 text-sm font-bold">← กลับ</a></div></body></html>"#.into());
    }
    let stats_res = app.ai.clone().find_all(FindAllRequest { collection: "stats".into() }).await.ok();
    let econ_res = app.ai.clone().find_all(FindAllRequest { collection: "economy".into() }).await.ok();
    let rpg_res = app.ai.clone().find_all(FindAllRequest { collection: "rpg".into() }).await.ok();
    let total_users = stats_res.as_ref().map(|r| r.get_ref().items_json.len()).unwrap_or(0);
    let total_econ = econ_res.as_ref().map(|r| r.get_ref().items_json.len()).unwrap_or(0);
    let total_rpg = rpg_res.as_ref().map(|r| r.get_ref().items_json.len()).unwrap_or(0);
    let total_msgs: i64 = stats_res.as_ref().map(|r| {
        r.get_ref().items_json.iter().filter_map(|s| serde_json::from_str::<Value>(s).ok())
            .filter_map(|v| v.get("message_count").and_then(|v| v.as_i64())).sum::<i64>()
    }).unwrap_or(0);
    admin_panel_html(&admin_pass, total_users, total_econ, total_rpg, total_msgs)
}

fn admin_login_page() -> Html<String> {
    Html(r#"<!DOCTYPE html><html lang='th'><head><meta charset='UTF-8'><meta name='viewport' content='width=device-width,initial-scale=1'><title>Admin Login</title><script src='https://cdn.tailwindcss.com'></script><link href='https://fonts.googleapis.com/css2?family=Orbitron:wght@700;900&family=Noto+Sans+Thai:wght@400;600&display=swap' rel='stylesheet'><style>*{font-family:'Noto Sans Thai',sans-serif}.font-display{font-family:'Orbitron',sans-serif}body{background:#0a0a0f}.bg-grid{background-image:linear-gradient(rgba(168,85,247,0.03) 1px,transparent 1px),linear-gradient(90deg,rgba(168,85,247,0.03) 1px,transparent 1px);background-size:60px 60px}.input-glow:focus{box-shadow:0 0 20px rgba(168,85,247,0.3),inset 0 0 20px rgba(168,85,247,0.05)}</style></head><body class='min-h-screen bg-grid flex items-center justify-center'><div class='w-full max-w-md mx-4'><div class='text-center mb-10'><div class='text-5xl mb-4'>⚡</div><h1 class='font-display font-black text-2xl' style='background:linear-gradient(135deg,#a855f7,#ec4899);-webkit-background-clip:text;-webkit-text-fill-color:transparent'>TAHBOT</h1><p class='text-gray-500 text-xs mt-2 uppercase tracking-[0.3em]'>Admin Command Center</p></div><div class='rounded-2xl p-8 border border-white/5' style='background:linear-gradient(135deg,rgba(15,15,25,0.9),rgba(10,10,18,0.95));backdrop-filter:blur(20px)'><form method='GET' class='space-y-6'><div><label class='block text-[10px] uppercase tracking-[0.2em] text-gray-500 mb-2'>รหัสผ่านแอดมิน</label><input type='password' name='pass' required class='w-full px-5 py-3.5 bg-white/[0.03] border border-white/10 rounded-xl text-gray-200 placeholder-gray-700 focus:outline-none input-glow focus:border-purple-500/50 transition-all' placeholder='••••••••'></div><button type='submit' class='group relative w-full py-3.5 rounded-xl font-bold text-white overflow-hidden transition-all hover:scale-[1.02]'><div class='absolute inset-0 bg-gradient-to-r from-purple-600 to-pink-600'></div><span class='relative'>🔓 LOGIN</span></button></form></div><div class='text-center mt-6'><a href="https://dashboard.themostpoordev.top" class='text-gray-600 text-xs hover:text-gray-400'>← Dashboard</a></div></div></body></html>"#.into())
}

fn admin_panel_html(_admin_pass: &str, total_users: usize, total_econ: usize, total_rpg: usize, total_msgs: i64) -> Html<String> {
    let bots = vec![("ต๊ะ","🤖"),("อาวัง","😤"),("เจ๊มุ่ง","👵"),("เสี่ยหนู","🐭"),("ผู้คุมกฎ","⚖️"),("มอด","🔧"),("โยฮัน","🎭")];
    let mut bot_opts = String::new();
    for (n, e) in &bots { bot_opts.push_str(&format!("<option value='{}'>{} {}</option>", n, e, n)); }
    let classes = vec!["Warrior","Mage","Rogue","Healer","Tank","Archer"];
    let mut class_opts = String::new();
    for c in &classes { class_opts.push_str(&format!("<option value='{}'>{}</option>", c, c)); }

    let html = format!(r#"<!DOCTYPE html><html lang='th'><head><meta charset='UTF-8'><meta name='viewport' content='width=device-width,initial-scale=1'><title>⚡ COMMAND CENTER</title><script src='https://cdn.tailwindcss.com'></script><link href='https://fonts.googleapis.com/css2?family=Orbitron:wght@400;700;900&family=Noto+Sans+Thai:wght@300;400;600;700&display=swap' rel='stylesheet'>{}</head><body class='min-h-screen bg-grid text-gray-200'><div class='fixed top-0 left-1/2 -translate-x-1/2 w-[800px] h-[400px] rounded-full opacity-20 blur-[120px] pointer-events-none' style='background:radial-gradient(ellipse,rgba(168,85,247,0.4),transparent)'></div><nav class='relative border-b border-white/5' style='background:rgba(10,10,15,0.8);backdrop-filter:blur(20px)'><div class='max-w-7xl mx-auto px-6 py-4 flex items-center justify-between'><div class='flex items-center gap-3'><div class='text-3xl'>⚡</div><div><h1 class='font-display font-black text-xl tracking-wider glow-text' style='background:linear-gradient(135deg,#a855f7,#ec4899);-webkit-background-clip:text;-webkit-text-fill-color:transparent'>COMMAND CENTER</h1><div class='flex items-center gap-1.5 text-[10px] text-gray-500 uppercase tracking-[0.2em]'><span class='pulse-dot w-1.5 h-1.5 rounded-full bg-green-500 inline-block'></span>System Online</div></div></div><a href='https://dashboard.themostpoordev.top' class='px-4 py-2 rounded-xl text-white text-sm font-bold border border-white/5 hover:border-white/10 transition-all' style='background:rgba(255,255,255,0.03)'>📊 Dashboard</a></div></nav><main class='relative max-w-7xl mx-auto px-6 py-8 space-y-6'><div class='grid grid-cols-2 md:grid-cols-4 gap-4'><div class='card-glass rounded-2xl p-5'><div class='text-[10px] uppercase tracking-[0.15em] text-gray-500 mb-1'>👥 Users</div><div class='font-display font-bold text-2xl text-purple-300'>{}</div></div><div class='card-glass rounded-2xl p-5'><div class='text-[10px] uppercase tracking-[0.15em] text-gray-500 mb-1'>💬 Messages</div><div class='font-display font-bold text-2xl text-cyan-400'>{}</div></div><div class='card-glass rounded-2xl p-5'><div class='text-[10px] uppercase tracking-[0.15em] text-gray-500 mb-1'>💰 Economy</div><div class='font-display font-bold text-2xl text-amber-400'>{}</div></div><div class='card-glass rounded-2xl p-5'><div class='text-[10px] uppercase tracking-[0.15em] text-gray-500 mb-1'>⚔️ RPG</div><div class='font-display font-bold text-2xl text-green-400'>{}</div></div></div><div class='flex flex-wrap gap-2 border-b border-white/5 pb-4'><button onclick='showSec("event")' class='tab-btn active px-4 py-2.5 rounded-xl text-xs font-bold uppercase tracking-wider border border-white/5 text-gray-500 hover:text-gray-300 transition-all'>🎮 Events</button><button onclick='showSec("economy")' class='tab-btn px-4 py-2.5 rounded-xl text-xs font-bold uppercase tracking-wider border border-white/5 text-gray-500 hover:text-gray-300 transition-all'>💰 Economy</button><button onclick='showSec("rpg")' class='tab-btn px-4 py-2.5 rounded-xl text-xs font-bold uppercase tracking-wider border border-white/5 text-gray-500 hover:text-gray-300 transition-all'>⚔️ RPG</button><button onclick='showSec("broadcast")' class='tab-btn px-4 py-2.5 rounded-xl text-xs font-bold uppercase tracking-wider border border-white/5 text-gray-500 hover:text-gray-300 transition-all'>📢 Broadcast</button><button onclick='showSec("announce")' class='tab-btn px-4 py-2.5 rounded-xl text-xs font-bold uppercase tracking-wider border border-white/5 text-gray-500 hover:text-gray-300 transition-all'>📣 Announce</button><button onclick='showSec("users")' class='tab-btn px-4 py-2.5 rounded-xl text-xs font-bold uppercase tracking-wider border border-white/5 text-gray-500 hover:text-gray-300 transition-all'>👥 Users</button></div><section id='sec-event' class='section active'><div class='card-glass rounded-2xl p-8 border border-purple-500/10'><div class='flex items-center gap-3 mb-8'><div class='w-10 h-10 rounded-xl flex items-center justify-center text-xl' style='background:linear-gradient(135deg,rgba(168,85,247,0.2),rgba(236,72,153,0.2))'>🎮</div><div><div class='font-display font-bold text-lg tracking-wider text-gray-200'>START ROLE EVENT</div><div class='text-[10px] text-gray-600'>แจกยศให้คนที่พิมพ์ในช่อง</div></div></div><form action='/cmd' method='POST' class='space-y-6'><div class='grid grid-cols-1 md:grid-cols-2 gap-5'><div><label class='block text-[10px] uppercase tracking-[0.15em] text-gray-500 mb-2'>เลือกบอท</label><select name='bot_name' required class='select-custom w-full px-4 py-3 bg-white/[0.03] border border-white/10 rounded-xl text-gray-200 focus:outline-none input-glow focus:border-purple-500/50 transition-all'><option value='' disabled selected>เลือกบอท...</option>{}</select></div><div><label class='block text-[10px] uppercase tracking-[0.15em] text-gray-500 mb-2'>Channel ID</label><input type='text' name='channel_id' placeholder='149...' required class='w-full px-4 py-3 bg-white/[0.03] border border-white/10 rounded-xl text-gray-200 placeholder-gray-700 focus:outline-none input-glow font-mono text-sm'></div><div><label class='block text-[10px] uppercase tracking-[0.15em] text-gray-500 mb-2'>Role ID</label><input type='text' name='role_id' placeholder='149...' required class='w-full px-4 py-3 bg-white/[0.03] border border-white/10 rounded-xl text-gray-200 placeholder-gray-700 focus:outline-none input-glow font-mono text-sm'></div><div><label class='block text-[10px] uppercase tracking-[0.15em] text-gray-500 mb-2'>จำนวนคน (N)</label><input type='number' name='max_people' value='3' min='1' max='50' required class='w-full px-4 py-3 bg-white/[0.03] border border-white/10 rounded-xl text-gray-200 focus:outline-none input-glow font-mono text-sm'></div><div><label class='block text-[10px] uppercase tracking-[0.15em] text-gray-500 mb-2'>เวลา (นาที)</label><input type='number' name='minutes' value='3' min='1' max='120' required class='w-full px-4 py-3 bg-white/[0.03] border border-white/10 rounded-xl text-gray-200 focus:outline-none input-glow font-mono text-sm'></div><div><label class='block text-[10px] uppercase tracking-[0.15em] text-gray-500 mb-2'>รหัสผ่านแอดมิน</label><input type='password' name='pass' placeholder='••••••••' required class='w-full px-4 py-3 bg-white/[0.03] border border-white/10 rounded-xl text-gray-200 placeholder-gray-700 focus:outline-none input-glow'></div></div><button type='submit' class='group relative w-full py-4 rounded-xl font-bold text-white text-lg overflow-hidden transition-all hover:scale-[1.01]'><div class='absolute inset-0 bg-gradient-to-r from-purple-600 via-pink-600 to-purple-600'></div><span class='relative flex items-center justify-center gap-3'>🚀 START EVENT</span></button></form></div></section><section id='sec-economy' class='section'><div class='grid grid-cols-1 lg:grid-cols-2 gap-6'><div class='card-glass rounded-2xl p-6'><div class='flex items-center gap-3 mb-6'><div class='w-10 h-10 rounded-xl bg-amber-500/20 flex items-center justify-center text-xl'>💰</div><div><div class='font-display font-bold text-sm tracking-wider text-gray-200'>SET BALANCE</div><div class='text-[10px] text-gray-600'>แก้ไขเงินผู้เล่น (บันทึกลง MongoDB ทันที)</div></div></div><form action='/api/economy/set' method='POST' class='space-y-4'><div class='grid grid-cols-2 gap-4'><div><label class='block text-[10px] uppercase tracking-[0.15em] text-gray-500 mb-1'>User ID</label><input type='text' name='user_id' required class='w-full px-3 py-2.5 bg-white/[0.03] border border-white/10 rounded-lg text-gray-200 text-sm focus:outline-none input-glow font-mono' placeholder='user_id'></div><div><label class='block text-[10px] uppercase tracking-[0.15em] text-gray-500 mb-1'>Username</label><input type='text' name='username' required class='w-full px-3 py-2.5 bg-white/[0.03] border border-white/10 rounded-lg text-gray-200 text-sm focus:outline-none input-glow' placeholder='username'></div></div><div class='grid grid-cols-2 gap-4'><div><label class='block text-[10px] uppercase tracking-[0.15em] text-gray-500 mb-1'>Wallet</label><input type='number' name='wallet' value='0' class='w-full px-3 py-2.5 bg-white/[0.03] border border-white/10 rounded-lg text-gray-200 text-sm focus:outline-none input-glow font-mono'></div><div><label class='block text-[10px] uppercase tracking-[0.15em] text-gray-500 mb-1'>Title</label><input type='text' name='title' class='w-full px-3 py-2.5 bg-white/[0.03] border border-white/10 rounded-lg text-gray-200 text-sm focus:outline-none input-glow' placeholder='เจ้าพ่อ...'></div></div><div><label class='block text-[10px] uppercase tracking-[0.15em] text-gray-500 mb-1'>Admin Password</label><input type='password' name='pass' required class='w-full px-3 py-2.5 bg-white/[0.03] border border-white/10 rounded-lg text-gray-200 text-sm focus:outline-none input-glow' placeholder='••••••••'></div><button type='submit' class='w-full py-3 rounded-xl font-bold text-white text-sm bg-amber-600 hover:bg-amber-500 transition-colors'>💾 SAVE TO DATABASE</button></form></div><div class='card-glass rounded-2xl p-6'><div class='flex items-center gap-3 mb-6'><div class='w-10 h-10 rounded-xl bg-amber-500/20 flex items-center justify-center text-xl'>ℹ️</div><div><div class='font-display font-bold text-sm tracking-wider text-gray-200'>HOW IT WORKS</div><div class='text-[10px] text-gray-600'>ข้อมูลถูกบันทึกผ่าน gRPC</div></div></div><div class='space-y-3'><div class='p-4 rounded-xl bg-white/[0.02] border border-white/5'><div class='text-xs text-gray-400 mb-1'>🔄 Data Flow</div><div class='text-[11px] text-gray-600'>Web → ai-core (gRPC proxy) → db-manager → MongoDB (tah_economy.economy_players)</div></div><div class='p-4 rounded-xl bg-white/[0.02] border border-white/5'><div class='text-xs text-gray-400 mb-1'>📝 Fields</div><div class='text-[11px] text-gray-600'>user_id, username, wallet, title — ถูกเก็บทั้งหมดใน document</div></div></div></div></div></section><section id='sec-rpg' class='section'><div class='grid grid-cols-1 lg:grid-cols-2 gap-6'><div class='card-glass rounded-2xl p-6'><div class='flex items-center gap-3 mb-6'><div class='w-10 h-10 rounded-xl bg-cyan-500/20 flex items-center justify-center text-xl'>⚔️</div><div><div class='font-display font-bold text-sm tracking-wider text-gray-200'>SET RPG STATS</div><div class='text-[10px] text-gray-600'>แก้ไขข้อมูล RPG (บันทึกลง MongoDB ทันที)</div></div></div><form action='/api/rpg/set' method='POST' class='space-y-4'><div class='grid grid-cols-2 gap-4'><div><label class='block text-[10px] uppercase tracking-[0.15em] text-gray-500 mb-1'>User ID</label><input type='text' name='user_id' required class='w-full px-3 py-2.5 bg-white/[0.03] border border-white/10 rounded-lg text-gray-200 text-sm focus:outline-none input-glow font-mono' placeholder='user_id'></div><div><label class='block text-[10px] uppercase tracking-[0.15em] text-gray-500 mb-1'>Username</label><input type='text' name='username' required class='w-full px-3 py-2.5 bg-white/[0.03] border border-white/10 rounded-lg text-gray-200 text-sm focus:outline-none input-glow' placeholder='username'></div></div><div class='grid grid-cols-3 gap-4'><div><label class='block text-[10px] uppercase tracking-[0.15em] text-gray-500 mb-1'>Level</label><input type='number' name='level' value='1' min='1' max='9999' class='w-full px-3 py-2.5 bg-white/[0.03] border border-white/10 rounded-lg text-gray-200 text-sm focus:outline-none input-glow font-mono'></div><div><label class='block text-[10px] uppercase tracking-[0.15em] text-gray-500 mb-1'>Class</label><select name='class' class='select-custom w-full px-3 py-2.5 bg-white/[0.03] border border-white/10 rounded-lg text-gray-200 text-sm focus:outline-none input-glow'>{}</select></div><div><label class='block text-[10px] uppercase tracking-[0.15em] text-gray-500 mb-1'>EXP</label><input type='number' name='exp' value='0' min='0' class='w-full px-3 py-2.5 bg-white/[0.03] border border-white/10 rounded-lg text-gray-200 text-sm focus:outline-none input-glow font-mono'></div></div><div><label class='block text-[10px] uppercase tracking-[0.15em] text-gray-500 mb-1'>Admin Password</label><input type='password' name='pass' required class='w-full px-3 py-2.5 bg-white/[0.03] border border-white/10 rounded-lg text-gray-200 text-sm focus:outline-none input-glow' placeholder='••••••••'></div><button type='submit' class='w-full py-3 rounded-xl font-bold text-white text-sm bg-cyan-600 hover:bg-cyan-500 transition-colors'>💾 SAVE TO DATABASE</button></form></div><div class='card-glass rounded-2xl p-6'><div class='flex items-center gap-3 mb-6'><div class='w-10 h-10 rounded-xl bg-cyan-500/20 flex items-center justify-center text-xl'>🎭</div><div><div class='font-display font-bold text-sm tracking-wider text-gray-200'>CLASS GUIDE</div><div class='text-[10px] text-gray-600'>ระบบ RPG</div></div></div><div class='space-y-2'><div class='flex items-center gap-3 p-3 rounded-lg bg-white/[0.02] border border-white/5'><span class='text-lg'>⚔️</span><div><div class='text-xs font-bold text-red-400'>Warrior</div><div class='text-[10px] text-gray-600'>นักรบ พลังโจมตีสูง</div></div></div><div class='flex items-center gap-3 p-3 rounded-lg bg-white/[0.02] border border-white/5'><span class='text-lg'>🔮</span><div><div class='text-xs font-bold text-blue-400'>Mage</div><div class='text-[10px] text-gray-600'>นักเวทย์</div></div></div><div class='flex items-center gap-3 p-3 rounded-lg bg-white/[0.02] border border-white/5'><span class='text-lg'>🗡️</span><div><div class='text-xs font-bold text-green-400'>Rogue</div><div class='text-[10px] text-gray-600'>นักฆ่า</div></div></div><div class='flex items-center gap-3 p-3 rounded-lg bg-white/[0.02] border border-white/5'><span class='text-lg'>💚</span><div><div class='text-xs font-bold text-yellow-400'>Healer</div><div class='text-[10px] text-gray-600'>นักรักษา</div></div></div><div class='flex items-center gap-3 p-3 rounded-lg bg-white/[0.02] border border-white/5'><span class='text-lg'>🛡️</span><div><div class='text-xs font-bold text-purple-400'>Tank</div><div class='text-[10px] text-gray-600'>แทงก์</div></div></div><div class='flex items-center gap-3 p-3 rounded-lg bg-white/[0.02] border border-white/5'><span class='text-lg'>🏹</span><div><div class='text-xs font-bold text-orange-400'>Archer</div><div class='text-[10px] text-gray-600'>นักธนู</div></div></div></div></div></div></section><section id='sec-announce' class='section'><div class='card-glass rounded-2xl p-8 border border-emerald-500/10'><div class='flex items-center gap-3 mb-8'><div class='w-10 h-10 rounded-xl flex items-center justify-center text-xl' style='background:linear-gradient(135deg,rgba(16,185,129,0.2),rgba(52,211,153,0.2))'>📣</div><div><div class='font-display font-bold text-lg tracking-wider text-gray-200'>CROSS-PLATFORM ANNOUNCE</div><div class='text-[10px] text-gray-600'>ส่งข้อความประกาศไปทุกแพลตฟอร์ม (Discord + LINE) ผ่าน gRPC</div></div></div><form action='/api/announce' method='POST' class='space-y-6'><div class='grid grid-cols-1 md:grid-cols-2 gap-5'><div><label class='block text-[10px] uppercase tracking-[0.15em] text-gray-500 mb-2'>เลือกบอท</label><select name='bot_name' required class='select-custom w-full px-4 py-3 bg-white/[0.03] border border-white/10 rounded-xl text-gray-200 focus:outline-none input-glow focus:border-purple-500/50 transition-all'><option value='' disabled selected>เลือกบอท...</option>{}</select></div><div><label class='block text-[10px] uppercase tracking-[0.15em] text-gray-500 mb-2'>Channel ID</label><input type='text' name='channel_id' placeholder='149...' required class='w-full px-4 py-3 bg-white/[0.03] border border-white/10 rounded-xl text-gray-200 placeholder-gray-700 focus:outline-none input-glow font-mono text-sm'></div></div><div><label class='block text-[10px] uppercase tracking-[0.15em] text-gray-500 mb-2'>ข้อความประกาศ</label><textarea name='message' rows='4' required class='w-full px-4 py-3 bg-white/[0.03] border border-white/10 rounded-xl text-gray-200 placeholder-gray-700 focus:outline-none input-glow text-sm' placeholder='พิมพ์ข้อความที่จะประกาศไปทุกแพลตฟอร์ม...'></textarea></div><div><label class='block text-[10px] uppercase tracking-[0.15em] text-gray-500 mb-2'>รหัสผ่านแอดมิน</label><input type='password' name='pass' required class='w-full px-4 py-3 bg-white/[0.03] border border-white/10 rounded-xl text-gray-200 placeholder-gray-700 focus:outline-none input-glow' placeholder='••••••••'></div><button type='submit' class='group relative w-full py-4 rounded-xl font-bold text-white text-lg overflow-hidden transition-all hover:scale-[1.01]'><div class='absolute inset-0 bg-gradient-to-r from-emerald-600 to-teal-600'></div><span class='relative flex items-center justify-center gap-3'>📣 SEND ANNOUNCE TO ALL PLATFORMS</span></button></form></div></section><section id='sec-broadcast' class='section'><div class='card-glass rounded-2xl p-8 border border-rose-500/10'><div class='flex items-center gap-3 mb-8'><div class='w-10 h-10 rounded-xl flex items-center justify-center text-xl' style='background:linear-gradient(135deg,rgba(244,63,94,0.2),rgba(251,146,60,0.2))'>📢</div><div><div class='font-display font-bold text-lg tracking-wider text-gray-200'>BROADCAST MESSAGE</div><div class='text-[10px] text-gray-600'>บันทึกข้อความไว้ใน config ให้บอทอ่านผ่าน web command queue</div></div></div><form action='/api/broadcast' method='POST' class='space-y-6'><div class='grid grid-cols-1 md:grid-cols-2 gap-5'><div><label class='block text-[10px] uppercase tracking-[0.15em] text-gray-500 mb-2'>เลือกบอท</label><select name='bot_name' required class='select-custom w-full px-4 py-3 bg-white/[0.03] border border-white/10 rounded-xl text-gray-200 focus:outline-none input-glow focus:border-purple-500/50 transition-all'><option value='' disabled selected>เลือกบอท...</option>{}</select></div><div><label class='block text-[10px] uppercase tracking-[0.15em] text-gray-500 mb-2'>Channel ID</label><input type='text' name='channel_id' placeholder='149...' required class='w-full px-4 py-3 bg-white/[0.03] border border-white/10 rounded-xl text-gray-200 placeholder-gray-700 focus:outline-none input-glow font-mono text-sm'></div></div><div><label class='block text-[10px] uppercase tracking-[0.15em] text-gray-500 mb-2'>ข้อความ</label><textarea name='message' rows='4' required class='w-full px-4 py-3 bg-white/[0.03] border border-white/10 rounded-xl text-gray-200 placeholder-gray-700 focus:outline-none input-glow text-sm' placeholder='พิมพ์ข้อความที่จะ broadcast...'></textarea></div><div><label class='block text-[10px] uppercase tracking-[0.15em] text-gray-500 mb-2'>รหัสผ่านแอดมิน</label><input type='password' name='pass' required class='w-full px-4 py-3 bg-white/[0.03] border border-white/10 rounded-xl text-gray-200 placeholder-gray-700 focus:outline-none input-glow' placeholder='••••••••'></div><button type='submit' class='group relative w-full py-4 rounded-xl font-bold text-white text-lg overflow-hidden transition-all hover:scale-[1.01]'><div class='absolute inset-0 bg-gradient-to-r from-rose-600 to-orange-600'></div><span class='relative flex items-center justify-center gap-3'>📢 SEND BROADCAST</span></button></form></div></section><section id='sec-users' class='section'><div class='card-glass rounded-2xl p-6'><div class='flex items-center gap-3 mb-6'><div class='w-10 h-10 rounded-xl bg-indigo-500/20 flex items-center justify-center text-xl'>👥</div><div><div class='font-display font-bold text-sm tracking-wider text-gray-200'>DATABASE STATUS</div><div class='text-[10px] text-gray-600'>ข้อมูลจริงจาก MongoDB</div></div></div><div class='grid grid-cols-2 md:grid-cols-4 gap-4 mb-6'><div class='p-4 rounded-xl bg-white/[0.02] border border-white/5 text-center'><div class='text-2xl font-display font-bold text-purple-300'>{}</div><div class='text-[10px] text-gray-500 mt-1'>Users</div></div><div class='p-4 rounded-xl bg-white/[0.02] border border-white/5 text-center'><div class='text-2xl font-display font-bold text-cyan-400'>{}</div><div class='text-[10px] text-gray-500 mt-1'>Messages</div></div><div class='p-4 rounded-xl bg-white/[0.02] border border-white/5 text-center'><div class='text-2xl font-display font-bold text-amber-400'>{}</div><div class='text-[10px] text-gray-500 mt-1'>Economy</div></div><div class='p-4 rounded-xl bg-white/[0.02] border border-white/5 text-center'><div class='text-2xl font-display font-bold text-green-400'>{}</div><div class='text-[10px] text-gray-500 mt-1'>RPG</div></div></div><div class='p-4 rounded-xl bg-white/[0.02] border border-white/5'><div class='text-xs text-gray-400 mb-2'>📡 API Endpoints</div><div class='space-y-1.5 text-[11px] font-mono text-gray-600'><div><span class='text-green-400'>GET</span> <a href='/api/user/test123' class='text-purple-400 hover:underline'>/api/user/&lt;id&gt;</a> — ดึงข้อมูลผู้เล่น</div><div><span class='text-amber-400'>POST</span> /api/economy/set — แก้ไขเงิน</div><div><span class='text-cyan-400'>POST</span> /api/rpg/set — แก้ไข RPG</div><div><span class='text-rose-400'>POST</span> /api/broadcast — ส่งข้อความ</div></div></div></div></section></main><footer class='relative text-center py-8 text-gray-700 text-xs border-t border-white/5 mt-8'><div class='flex items-center justify-center gap-4'><span>TAHBOT ENGINE v2.0</span><span class='w-1 h-1 rounded-full bg-gray-700'></span><span class='text-purple-400'>ADMIN ONLY</span></div></footer><script>function showSec(id){{document.querySelectorAll('.section').forEach(e=>e.classList.remove('active'));document.getElementById('sec-'+id).classList.add('active');document.querySelectorAll('.tab-btn').forEach(e=>e.classList.remove('active'));event.target.classList.add('active')}}</script></body></html>"#,
        admin_styles(),
        total_users, total_msgs, total_econ, total_rpg,
        bot_opts, class_opts, bot_opts, bot_opts,
        total_users, total_msgs, total_econ, total_rpg
    );
    Html(html)
}

pub async fn admin_post_handler(State(app): State<Arc<AppState>>, Form(payload): Form<crate::AdminCmd>) -> Html<String> {
    if !verify_admin(&payload.pass) {
        return Html(r#"<script>alert("รหัสผิด!");history.back()</script>"#.into());
    }

    let channel_id: u64 = payload.channel_id.parse().unwrap_or(0);
    let role_id: u64 = payload.role_id.parse().unwrap_or(0);

    let task = crate::AdminTask {
        bot_name: payload.bot_name.clone(),
        channel_id,
        role_id,
        max_people: payload.max_people,
        end_time: Instant::now() + Duration::from_secs(payload.minutes * 60),
        participants: Arc::new(tokio::sync::Mutex::new(HashSet::<u64>::new())),
    };
    *app.active_task.lock().await = Some(task);

    let admin_pass = std::env::var("ADMIN_PASS").unwrap_or_default();
    Html(format!(r#"<!DOCTYPE html><html><head><script src="https://cdn.tailwindcss.com"></script></head><body class="bg-[#0a0a0f] min-h-screen flex items-center justify-center"><div class="text-center"><div class="text-8xl mb-6">✅</div><h1 class="text-4xl font-black text-green-400 mb-3">EVENT STARTED!</h1><p class="text-gray-300 text-lg mb-2">บอท <span class="font-bold text-purple-300">{}</span> กำลังแจกยศแล้ว</p><p class="text-gray-600 text-sm mb-8">ผู้ใช้ที่พิมพ์ในช่องจะได้รับยศทันที</p><a href="https://admin.themostpoordev.top?pass={}" class="px-8 py-4 rounded-xl font-bold text-white text-lg" style="background:linear-gradient(135deg,#9333ea,#db2777)">⚡ กลับไป Master Control</a></div></body></html>"#, payload.bot_name, admin_pass))
}

pub async fn api_get_user(State(app): State<Arc<AppState>>, axum::extract::Path(user_id): axum::extract::Path<String>) -> Result<Html<String>, StatusCode> {
    let stats = app.ai.clone().find_all(FindAllRequest { collection: "stats".into() }).await.ok();
    let economy = app.ai.clone().find_all(FindAllRequest { collection: "economy".into() }).await.ok();
    let rpg = app.ai.clone().find_all(FindAllRequest { collection: "rpg".into() }).await.ok();
    let mut data = serde_json::json!({"user_id": user_id});
    if let Some(resp) = &stats {
        for s in &resp.get_ref().items_json {
            if let Ok(v) = serde_json::from_str::<Value>(s) {
                if v.get("user_id").and_then(|v| v.as_str()) == Some(&user_id) { data["stats"] = v; break; }
            }
        }
    }
    if let Some(resp) = &economy {
        for s in &resp.get_ref().items_json {
            if let Ok(v) = serde_json::from_str::<Value>(s) {
                if v.get("user_id").and_then(|v| v.as_str()) == Some(&user_id) { data["economy"] = v; break; }
            }
        }
    }
    if let Some(resp) = &rpg {
        for s in &resp.get_ref().items_json {
            if let Ok(v) = serde_json::from_str::<Value>(s) {
                if v.get("user_id").and_then(|v| v.as_str()) == Some(&user_id) { data["rpg"] = v; break; }
            }
        }
    }
    let pretty = serde_json::to_string_pretty(&data).unwrap_or_default();
    Ok(Html(format!(r#"<!DOCTYPE html><html><head><script src="https://cdn.tailwindcss.com"></script></head><body class="bg-[#0a0a0f] min-h-screen p-8"><pre class="text-green-400 text-sm font-mono bg-black/40 p-6 rounded-xl border border-white/5 overflow-auto">{}</pre></body></html>"#, pretty)))
}

pub async fn api_set_economy(State(app): State<Arc<AppState>>, Form(form): Form<SetEconomyForm>) -> Html<String> {
    if !verify_admin(&form.pass) {
        return Html(r#"<script>alert("รหัสผิด!");history.back()</script>"#.into());
    }
    let data = serde_json::json!({"wallet": form.wallet, "title": form.title});
    match app.ai.clone().upsert_economy(UpsertEconomyRequest { user_id: form.user_id, username: form.username, data_json: data.to_string() }).await {
        Ok(_) => Html(r#"<script>alert("✅ บันทึกเรียบร้อย!");history.back()</script>"#.into()),
        Err(e) => Html(format!(r#"<script>alert("❌ ผิดพลาด: {}");history.back()</script>"#, e)),
    }
}

pub async fn api_set_rpg(State(app): State<Arc<AppState>>, Form(form): Form<SetRpgForm>) -> Html<String> {
    if !verify_admin(&form.pass) {
        return Html(r#"<script>alert("รหัสผิด!");history.back()</script>"#.into());
    }
    let existing = app.ai.clone().get_rpg(GetRpgRequest { user_id: form.user_id.clone() }).await.ok();
    let mut doc: serde_json::Value = existing.and_then(|r| serde_json::from_str(&r.into_inner().data_json).ok()).unwrap_or_else(|| serde_json::json!({}));
    if let Some(obj) = doc.as_object_mut() {
        obj.insert("user_id".into(), serde_json::json!(form.user_id.clone()));
        obj.insert("username".into(), serde_json::json!(form.username.clone()));
        obj.insert("class".into(), serde_json::json!(form.class.clone()));
        obj.insert("level".into(), serde_json::json!(form.level));
        obj.insert("exp".into(), serde_json::json!(form.exp));
    }
    match app.ai.clone().upsert_rpg(UpsertRpgRequest { user_id: form.user_id, username: form.username, class: form.class, data_json: serde_json::to_string(&doc).unwrap() }).await {
        Ok(_) => Html(r#"<script>alert("✅ บันทึกเรียบร้อย!");history.back()</script>"#.into()),
        Err(e) => Html(format!(r#"<script>alert("❌ ผิดพลาด: {}");history.back()</script>"#, e)),
    }
}

pub async fn api_broadcast(State(app): State<Arc<AppState>>, Form(form): Form<BroadcastForm>) -> Html<String> {
    if !verify_admin(&form.pass) {
        return Html(r#"<script>alert("รหัสผิด!");history.back()</script>"#.into());
    }
    let command_id = format!("broadcast_{}", chrono::Utc::now().timestamp_millis());
    let payload = serde_json::json!({
        "bot_name": form.bot_name,
        "channel_id": form.channel_id,
        "message": form.message,
    });
    match app.ai.clone().insert_web_command(InsertWebCommandRequest {
        command_id,
        command_type: "broadcast".into(),
        payload_json: payload.to_string(),
        status: "pending".into(),
        created_at: chrono::Utc::now().timestamp(),
    }).await {
        Ok(_) => Html(r#"<script>alert("📢 Broadcast queued! Bot will pick it up shortly.");history.back()</script>"#.into()),
        Err(e) => Html(format!(r#"<script>alert("❌ ผิดพลาด: {}");history.back()</script>"#, e)),
    }
}

pub async fn api_announce(State(app): State<Arc<AppState>>, Form(form): Form<AnnounceForm>) -> Html<String> {
    if !verify_admin(&form.pass) {
        return Html(r#"<script>alert("รหัสผิด!");history.back()</script>"#.into());
    }
    let command_id = format!("announce_{}", chrono::Utc::now().timestamp_millis());
    let payload = serde_json::json!({
        "bot_name": form.bot_name,
        "channel_id": form.channel_id,
        "message": form.message,
    });
    match app.ai.clone().insert_web_command(InsertWebCommandRequest {
        command_id,
        command_type: "announce".into(),
        payload_json: payload.to_string(),
        status: "pending".into(),
        created_at: chrono::Utc::now().timestamp(),
    }).await {
        Ok(_) => Html(r#"<script>alert("📣 Announce queued! Bot will pick it up shortly.");history.back()</script>"#.into()),
        Err(e) => Html(format!(r#"<script>alert("❌ ผิดพลาด: {}");history.back()</script>"#, e)),
    }
}
