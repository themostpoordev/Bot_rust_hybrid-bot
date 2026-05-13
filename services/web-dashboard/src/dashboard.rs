use axum::{extract::State, response::Html};
use crate::bot_messaging::*;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

use crate::AppState;

fn tier_name(rude: i32, lewd: i32) -> &'static str {
    match rude + lewd { 0..=10 => "ปลอดภัย", 11..=30 => "เริ่มแหก", 31..=60 => "สายถ่อย", 61..=100 => "ตัวร้าย", 101..=200 => "จอมวายร้าย", _ => "ระดับนรก" }
}
fn tier_color(rude: i32, lewd: i32) -> &'static str {
    match rude + lewd { 0..=10 => "#22c55e", 11..=30 => "#eab308", 31..=60 => "#f97316", 61..=100 => "#ef4444", 101..=200 => "#a855f7", _ => "#ec4899" }
}
fn progress_bar(value: i32, max: i32, color: &str) -> String {
    let pct = ((value as f64 / max as f64) * 100.0).min(100.0) as i32;
    format!("<div class='w-full bg-gray-800/60 rounded-full h-2.5 overflow-hidden'><div class='h-full rounded-full' style='width:{}%;background:{};box-shadow:0 0 10px {}80'></div></div>", pct, color, color)
}

const STYLE: &str = r#"*{font-family:'Noto Sans Thai',sans-serif}.font-display{font-family:'Orbitron',sans-serif}body{background:#0a0a0f}.bg-grid{background-image:linear-gradient(rgba(168,85,247,0.03) 1px,transparent 1px),linear-gradient(90deg,rgba(168,85,247,0.03) 1px,transparent 1px);background-size:60px 60px}.glow-text{text-shadow:0 0 40px rgba(168,85,247,0.5),0 0 80px rgba(168,85,247,0.2)}.card-glass{background:linear-gradient(135deg,rgba(15,15,25,0.8),rgba(10,10,18,0.9));border:1px solid rgba(255,255,255,0.05);backdrop-filter:blur(20px)}.pulse-dot{animation:pulse 2s infinite}@keyframes pulse{0%,100%{opacity:1}50%{opacity:0.4}}@keyframes statusPulse{0%,100%{transform:scale(1);opacity:0.4}50%{transform:scale(1.5);opacity:0}}.status-ping{animation:statusPulse 2s ease-in-out infinite}@keyframes float{0%,100%{transform:translateY(0)}50%{transform:translateY(-6px)}}.float{animation:float 4s ease-in-out infinite}.tab-content{display:none}.tab-content.active{display:block}"#;

const SCRIPT: &str = r#"
function showTab(t){
  document.querySelectorAll('.tab-content').forEach(e=>e.classList.remove('active'));
  document.getElementById('tab-'+t).classList.add('active');
  document.querySelectorAll('a[onclick^=showTab]').forEach(e=>{
    e.classList.remove('bg-purple-500/20','text-purple-300','border-purple-500/30');
    e.classList.add('text-gray-500','border-transparent');
  });
  event.target.classList.add('bg-purple-500/20','text-purple-300','border-purple-500/30');
  event.target.classList.remove('text-gray-500','border-transparent');
}
function formatLatency(us){
  if(us<1000)return us+'µs';
  return(us/1000).toFixed(1)+'ms';
}
function updateStatusCard(card,name,status,latencyUs){
  const isOnline=status==='online';
  const indicator=card.querySelector('.relative > .w-3');
  const ping=card.querySelector('.status-ping');
  const icon=card.querySelector('.w-10');
  const text=card.querySelector('.status-text');
  const latency=card.querySelector('.status-latency');
  const statusLabel=isOnline?(name==='db-manager'?'Connected':name==='web-dashboard'?'Active':'Online'):'Offline';
  const latencyText=isOnline?formatLatency(latencyUs):'—';
  text.textContent=isOnline?statusLabel:'Offline';
  latency.textContent=isOnline?(latencyUs>0?latencyText:'<1ms'):'—';
  if(isOnline){
    indicator.className='w-3 h-3 rounded-full bg-green-500';
    ping.className='status-ping absolute inset-0 w-3 h-3 rounded-full bg-green-500/30';
    icon.className='w-10 h-10 rounded-xl bg-green-500/10 border border-green-500/15 flex items-center justify-center text-green-400 text-lg';
    text.className='text-[10px] text-green-400 uppercase tracking-wider font-semibold status-text';
  } else {
    indicator.className='w-3 h-3 rounded-full bg-red-500';
    ping.className='status-ping absolute inset-0 w-3 h-3 rounded-full bg-red-500/30';
    icon.className='w-10 h-10 rounded-xl bg-red-500/10 border border-red-500/15 flex items-center justify-center text-red-400 text-lg';
    text.className='text-[10px] text-red-400 uppercase tracking-wider font-semibold status-text';
  }
}
async function pollHealth(){
  try {
    const resp=await fetch('/health');
    if(!resp.ok)throw new Error('HTTP '+resp.status);
    const data=await resp.json();
    if(data.services){
      for(const svc of data.services){
        const card=document.querySelector('.status-card[data-service="'+svc.name+'"]');
        if(card)updateStatusCard(card,svc.name,svc.status,svc.latency_us);
      }
    }
  } catch(e) {
    ['ai-core','db-manager','web-dashboard'].forEach(name=>{
      const card=document.querySelector('.status-card[data-service="'+name+'"]');
      if(card)updateStatusCard(card,name,'offline',0);
    });
  }
}
pollHealth();
setInterval(pollHealth,15000);
"#;

fn status_card_html() -> String {
    let services = [
        ("ai-core", "🧠", "AI Bot Core"),
        ("db-manager", "🗄️", "Database Manager"),
        ("web-dashboard", "🌐", "Web Dashboard"),
    ];
    let mut html = String::new();
    for (name, icon, label) in &services {
        html.push_str(&format!(
            "<div class='status-card p-5' data-service='{}'><div class='flex items-center gap-4'><div class='relative'><div class='w-3 h-3 rounded-full bg-green-500'></div><div class='status-ping absolute inset-0 w-3 h-3 rounded-full bg-green-500/30'></div></div><div class='w-10 h-10 rounded-xl bg-green-500/10 border border-green-500/15 flex items-center justify-center text-green-400 text-lg'>{}</div><div class='flex-1'><div class='font-bold text-sm text-gray-200'>{}</div><div class='text-[10px] text-green-400 uppercase tracking-wider font-semibold status-text'>Checking...</div></div><div class='text-right'><div class='font-mono text-[10px] text-gray-500 status-latency'>—</div></div></div></div>",
            name, icon, label
        ));
    }
    html
}

pub async fn dashboard_handler(State(app): State<Arc<AppState>>) -> Html<String> {
    let mut ai1 = app.ai.clone();
    let mut ai2 = app.ai.clone();
    let mut ai3 = app.ai.clone();
    let stats_f = ai1.find_all(FindAllRequest { collection: "stats".into() });
    let econ_f = ai2.find_all(FindAllRequest { collection: "economy".into() });
    let rpg_f = ai3.find_all(FindAllRequest { collection: "rpg".into() });
    let (stats_res, econ_res, rpg_res) = tokio::join!(stats_f, econ_f, rpg_f);

    // Parse stats
    let mut users: Vec<(String, String, i32, i32, i32)> = Vec::new();
    if let Err(ref e) = stats_res {
        eprintln!("[dashboard] stats find_all error: {}", e);
    }
    if let Ok(resp) = stats_res {
        eprintln!("[dashboard] stats items: {}", resp.get_ref().items_json.len());
        for s in &resp.into_inner().items_json {
            if let Ok(v) = serde_json::from_str::<Value>(s) {
                let uid = v.get("user_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
                if uid.is_empty() { continue; }
                let uname = v.get("username").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();
                let rude = v.get("rude_score").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                let lewd = v.get("lewd_score").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                let msgs = v.get("message_count").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                users.push((uid, uname, rude, lewd, msgs));
            }
        }
    }
    users.sort_by(|a, b| (b.2 + b.3).cmp(&(a.2 + a.3)));
    let total_users = users.len();
    let total_msgs: i32 = users.iter().map(|u| u.4).sum();
    let top_rude = users.iter().max_by_key(|u| u.2).map(|u| u.1.clone()).unwrap_or_else(|| "—".into());
    let top_lewd = users.iter().max_by_key(|u| u.3).map(|u| u.1.clone()).unwrap_or_else(|| "—".into());
    let top_tim = users.iter().min_by_key(|u| u.2 + u.3).map(|u| u.1.clone()).unwrap_or_else(|| "—".into());
    let max_score = users.iter().map(|u| u.2 + u.3).max().unwrap_or(1).max(1);

    let mut rows = String::new();
    for (i, u) in users.iter().enumerate() {
        let total = u.2 + u.3;
        let medal = match i { 0 => "🥇", 1 => "🥈", 2 => "🥉", _ => "" };
        let badge = if i < 3 {
            format!("<span class='inline-flex items-center justify-center w-8 h-8 rounded-full text-lg'>{}</span>", medal)
        } else {
            format!("<span class='inline-flex items-center justify-center w-8 h-8 rounded-full bg-gray-800/60 text-gray-500 text-sm font-bold'>{}</span>", i + 1)
        };
        let tier = tier_color(u.2, u.3);
        let glow = if i == 0 { "box-shadow:0 0 30px rgba(168,85,247,0.15);border-color:rgba(168,85,247,0.4);" } else { "" };
        rows.push_str(&format!(
            "<tr class='group hover:bg-white/[0.03] transition-all' style='{}'><td class='py-4 px-5'>{}</td><td class='py-4 px-5'><div class='flex items-center gap-3'><div class='w-9 h-9 rounded-xl bg-gradient-to-br from-purple-500/20 to-pink-500/20 flex items-center justify-center text-sm font-bold text-gray-300 border border-white/5'>{}</div><div><div class='font-semibold text-gray-200 text-sm'>{}</div><div class='text-[10px] uppercase tracking-wider font-bold' style='color:{}'>{}</div></div></div></td><td class='py-4 px-5'><div class='font-mono font-bold text-red-400 text-sm'>{}</div><div class='mt-1'>{}</div></td><td class='py-4 px-5'><div class='font-mono font-bold text-pink-400 text-sm'>{}</div><div class='mt-1'>{}</div></td><td class='py-4 px-5'><div class='font-mono font-bold text-sm' style='color:{};text-shadow:0 0 20px {}80'>⚡ {}</div></td><td class='py-4 px-5 text-gray-600 text-xs font-mono'>{}</td></tr>",
            glow, badge,
            u.1.chars().next().unwrap_or('?').to_uppercase().to_string(), u.1, tier, tier_name(u.2, u.3),
            u.2, progress_bar(u.2, max_score, "#ef4444"),
            u.3, progress_bar(u.3, max_score, "#ec4899"),
            tier, tier, total, u.4
        ));
    }
    if rows.is_empty() {
        rows = "<tr><td colspan='6' class='py-20 text-center text-gray-600'><div class='text-5xl mb-4'>📭</div><div class='text-lg'>ยังไม่มีข้อมูล</div></td></tr>".to_string();
    }

    // Parse economy
    let mut econ_users: Vec<(String, String, i32)> = Vec::new();
    if let Err(ref e) = econ_res {
        eprintln!("[dashboard] economy find_all error: {}", e);
    }
    if let Ok(resp) = econ_res {
        eprintln!("[dashboard] economy items: {}", resp.get_ref().items_json.len());
        for s in &resp.into_inner().items_json {
            if let Ok(v) = serde_json::from_str::<Value>(s) {
                let uid = v.get("user_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
                if uid.is_empty() { continue; }
                let uname = v.get("username").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();
                let bal = v.get("wallet").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                econ_users.push((uid, uname, bal));
            }
        }
    }
    econ_users.sort_by(|a, b| b.2.cmp(&a.2));
    let total_econ = econ_users.len();

    let mut econ_rows = String::new();
    for (i, u) in econ_users.iter().enumerate() {
        let medal = match i { 0 => "🥇", 1 => "🥈", 2 => "🥉", _ => "" };
        let badge = if i < 3 {
            format!("<span class='inline-flex items-center justify-center w-8 h-8 rounded-full text-lg'>{}</span>", medal)
        } else {
            format!("<span class='inline-flex items-center justify-center w-8 h-8 rounded-full bg-gray-800/60 text-gray-500 text-sm font-bold'>{}</span>", i + 1)
        };
        let bal_color = if u.2 >= 10000 { "text-amber-400" } else if u.2 >= 1000 { "text-green-400" } else { "text-gray-300" };
        econ_rows.push_str(&format!(
            "<tr class='hover:bg-white/[0.03] transition-all'><td class='py-4 px-5'>{}</td><td class='py-4 px-5'><div class='flex items-center gap-3'><div class='w-9 h-9 rounded-xl bg-gradient-to-br from-amber-500/20 to-yellow-500/20 flex items-center justify-center text-sm font-bold text-gray-300 border border-white/5'>{}</div><div class='font-semibold text-gray-200 text-sm'>{}</div></div></td><td class='py-4 px-5'><div class='font-mono font-bold {} text-sm'>💎 {}</div></td></tr>",
            badge, u.1.chars().next().unwrap_or('?').to_uppercase().to_string(), u.1, bal_color, u.2
        ));
    }
    if econ_rows.is_empty() {
        econ_rows = "<tr><td colspan='3' class='py-20 text-center text-gray-600'><div class='text-5xl mb-4'>💰</div><div>ยังไม่มีข้อมูล</div></td></tr>".to_string();
    }

    // Parse RPG
    let mut rpg_users: Vec<(String, String, i32, String, i32)> = Vec::new();
    if let Err(ref e) = rpg_res {
        eprintln!("[dashboard] rpg find_all error: {}", e);
    }
    if let Ok(resp) = rpg_res {
        eprintln!("[dashboard] rpg items: {}", resp.get_ref().items_json.len());
        for s in &resp.into_inner().items_json {
            if let Ok(v) = serde_json::from_str::<Value>(s) {
                let uid = v.get("user_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
                if uid.is_empty() { continue; }
                let uname = v.get("username").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();
                let lvl = v.get("level").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                let class = v.get("class").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();
                let exp = v.get("exp").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                rpg_users.push((uid, uname, lvl, class, exp));
            }
        }
    }
    rpg_users.sort_by(|a, b| b.2.cmp(&a.2).then(b.4.cmp(&a.4)));
    let total_rpg = rpg_users.len();

    let class_colors: HashMap<&str, &str> = [("warrior","#ef4444"),("mage","#3b82f6"),("rogue","#22c55e"),("healer","#eab308"),("tank","#8b5cf6"),("archer","#f97316")].iter().cloned().collect();
    let mut rpg_rows = String::new();
    for (i, u) in rpg_users.iter().enumerate() {
        let medal = match i { 0 => "🥇", 1 => "🥈", 2 => "🥉", _ => "" };
        let badge = if i < 3 {
            format!("<span class='inline-flex items-center justify-center w-8 h-8 rounded-full text-lg'>{}</span>", medal)
        } else {
            format!("<span class='inline-flex items-center justify-center w-8 h-8 rounded-full bg-gray-800/60 text-gray-500 text-sm font-bold'>{}</span>", i + 1)
        };
        let cc = class_colors.get(u.3.to_lowercase().as_str()).unwrap_or(&"#a855f7");
        rpg_rows.push_str(&format!(
            "<tr class='hover:bg-white/[0.03] transition-all'><td class='py-4 px-5'>{}</td><td class='py-4 px-5'><div class='flex items-center gap-3'><div class='w-9 h-9 rounded-xl flex items-center justify-center text-sm font-bold text-gray-300 border border-white/5' style='background:linear-gradient(135deg,{}20,{}10)'>{}</div><div><div class='font-semibold text-gray-200 text-sm'>{}</div><div class='text-[10px] uppercase tracking-wider font-bold' style='color:{}'>⚔️ {}</div></div></div></td><td class='py-4 px-5'><div class='font-mono font-bold text-cyan-400 text-sm'>Lv.{}</div><div class='mt-1'>{}</div></td><td class='py-4 px-5 text-gray-600 text-xs font-mono'>EXP {}</td></tr>",
            badge, cc, cc, u.1.chars().next().unwrap_or('?').to_uppercase().to_string(), u.1, cc, u.3, u.2, progress_bar(u.2, 100, cc), u.4
        ));
    }
    if rpg_rows.is_empty() {
        rpg_rows = "<tr><td colspan='4' class='py-20 text-center text-gray-600'><div class='text-5xl mb-4'>⚔️</div><div>ยังไม่มีข้อมูล</div></td></tr>".to_string();
    }

    let status_cards = status_card_html();

    let html = format!(
        "<!DOCTYPE html><html lang='th'><head><meta charset='UTF-8'><meta name='viewport' content='width=device-width,initial-scale=1'><title>🏆 TAHBOT LEAGUE</title><script src='https://cdn.tailwindcss.com'></script><link href='https://fonts.googleapis.com/css2?family=Orbitron:wght@400;700;900&family=Noto+Sans+Thai:wght@300;400;600;700&display=swap' rel='stylesheet'><style>{}</style></head><body class='min-h-screen bg-grid text-gray-200'><div class='fixed top-0 left-1/2 -translate-x-1/2 w-[800px] h-[400px] rounded-full opacity-20 blur-[120px] pointer-events-none' style='background:radial-gradient(ellipse,rgba(168,85,247,0.4),transparent)'></div><nav class='relative border-b border-white/5' style='background:rgba(10,10,15,0.8);backdrop-filter:blur(20px)'><div class='max-w-7xl mx-auto px-6 py-4 flex items-center justify-between'><div class='flex items-center gap-3'><div class='text-3xl float'>🏆</div><div><h1 class='font-display font-black text-xl tracking-wider glow-text' style='background:linear-gradient(135deg,#a855f7,#ec4899);-webkit-background-clip:text;-webkit-text-fill-color:transparent'>TAHBOT LEAGUE</h1><div class='flex items-center gap-1.5 text-[10px] text-gray-500 uppercase tracking-[0.2em]'><span class='pulse-dot w-1.5 h-1.5 rounded-full bg-green-500 inline-block'></span>Live · gRPC → MongoDB</div></div></div><a href='https://admin.themostpoordev.top' class='group relative px-5 py-2.5 rounded-xl text-sm font-bold text-white overflow-hidden transition-all hover:scale-105'><div class='absolute inset-0 bg-gradient-to-r from-purple-600 to-pink-600 opacity-80 group-hover:opacity-100 transition-opacity'></div><span class='relative'>⚡ ADMIN</span></a></div></nav><main class='relative max-w-7xl mx-auto px-6 py-8 space-y-6'><div class='grid grid-cols-2 md:grid-cols-4 gap-4'><div class='card-glass rounded-2xl p-5'><div class='text-[10px] uppercase tracking-[0.15em] text-gray-500 mb-1'>👥 Users</div><div class='font-display font-bold text-2xl text-white'>{}</div></div><div class='card-glass rounded-2xl p-5'><div class='text-[10px] uppercase tracking-[0.15em] text-gray-500 mb-1'>💬 Messages</div><div class='font-display font-bold text-2xl text-cyan-400'>{}</div></div><div class='card-glass rounded-2xl p-5'><div class='text-[10px] uppercase tracking-[0.15em] text-gray-500 mb-1'>💰 Economy</div><div class='font-display font-bold text-2xl text-amber-400'>{}</div></div><div class='card-glass rounded-2xl p-5'><div class='text-[10px] uppercase tracking-[0.15em] text-gray-500 mb-1'>⚔️ RPG</div><div class='font-display font-bold text-2xl text-green-400'>{}</div></div></div><div class='card-glass rounded-2xl overflow-hidden'><div class='px-6 py-4 border-b border-white/5 flex items-center justify-between'><div class='flex items-center gap-3'><div class='w-8 h-8 rounded-lg bg-green-500/20 flex items-center justify-center text-sm'>🖥️</div><div class='font-display font-bold text-sm tracking-wider text-gray-200'>SYSTEM STATUS</div></div><div class='flex items-center gap-1.5 text-[10px] text-green-400 uppercase tracking-wider'><span class='pulse-dot w-1.5 h-1.5 rounded-full bg-green-500 inline-block'></span>Live</div></div><div class='grid grid-cols-1 md:grid-cols-3 gap-0 divide-y md:divide-y-0 md:divide-x divide-white/5' id='statusGrid'>{}</div></div><div class='grid grid-cols-1 md:grid-cols-3 gap-4'><div class='card-glass rounded-2xl p-6 relative overflow-hidden'><div class='text-4xl mb-3'>🤬</div><div class='text-[10px] uppercase tracking-[0.2em] text-gray-500 mb-1'>สายถ่อยที่สุด</div><div class='font-display font-black text-2xl text-red-400 glow-text'>{}</div></div><div class='card-glass rounded-2xl p-6 relative overflow-hidden'><div class='text-4xl mb-3'>💦</div><div class='text-[10px] uppercase tracking-[0.2em] text-gray-500 mb-1'>สายกามที่สุด</div><div class='font-display font-black text-2xl text-pink-400 glow-text'>{}</div></div><div class='card-glass rounded-2xl p-6 relative overflow-hidden'><div class='text-4xl mb-3'>😇</div><div class='text-[10px] uppercase tracking-[0.2em] text-gray-500 mb-1'>ไอ้ติ๋มที่สุด</div><div class='font-display font-black text-2xl text-green-400 glow-text'>{}</div></div></div><div class='flex gap-2 border-b border-white/5 pb-4'><a href='#' onclick='showTab(\"stats\");return false' class='px-4 py-2 rounded-lg text-xs font-bold uppercase tracking-wider border bg-purple-500/20 text-purple-300 border-purple-500/30'>📊 Stats</a><a href='#' onclick='showTab(\"economy\");return false' class='px-4 py-2 rounded-lg text-xs font-bold uppercase tracking-wider border text-gray-500 border-transparent hover:text-gray-300'>💰 Economy</a><a href='#' onclick='showTab(\"rpg\");return false' class='px-4 py-2 rounded-lg text-xs font-bold uppercase tracking-wider border text-gray-500 border-transparent hover:text-gray-300'>⚔️ RPG</a></div><div id='tab-stats' class='tab-content active'><div class='card-glass rounded-2xl overflow-hidden'><div class='px-6 py-5 border-b border-white/5 flex items-center justify-between'><div class='flex items-center gap-3'><div class='w-8 h-8 rounded-lg bg-purple-500/20 flex items-center justify-center text-sm'>📊</div><div><div class='font-display font-bold text-sm tracking-wider text-gray-200'>LEADERBOARD</div><div class='text-[10px] text-gray-600'>{} players</div></div></div><div class='flex items-center gap-1.5 text-[10px] text-green-400 uppercase tracking-wider'><span class='pulse-dot w-1.5 h-1.5 rounded-full bg-green-500 inline-block'></span>Live</div></div><div class='overflow-x-auto'><table class='w-full text-sm'><thead class='text-gray-500 uppercase text-[10px] tracking-[0.15em] border-b border-white/5'><tr><th class='py-3 px-5 text-left'>Rank</th><th class='py-3 px-5 text-left'>Player</th><th class='py-3 px-5 text-left'><span class='text-red-400'>🤬 Rude</span></th><th class='py-3 px-5 text-left'><span class='text-pink-400'>💦 Lewd</span></th><th class='py-3 px-5 text-left'>Total</th><th class='py-3 px-5 text-left'>Msgs</th></tr></thead><tbody class='divide-y divide-white/[0.03]'>{}</tbody></table></div></div></div><div id='tab-economy' class='tab-content'><div class='card-glass rounded-2xl overflow-hidden'><div class='px-6 py-5 border-b border-white/5'><div class='flex items-center gap-3'><div class='w-8 h-8 rounded-lg bg-amber-500/20 flex items-center justify-center text-sm'>💰</div><div><div class='font-display font-bold text-sm tracking-wider text-gray-200'>ECONOMY</div><div class='text-[10px] text-gray-600'>{} players</div></div></div></div><div class='overflow-x-auto'><table class='w-full text-sm'><thead class='text-gray-500 uppercase text-[10px] tracking-[0.15em] border-b border-white/5'><tr><th class='py-3 px-5 text-left'>Rank</th><th class='py-3 px-5 text-left'>Player</th><th class='py-3 px-5 text-left'><span class='text-amber-400'>💎 Balance</span></th></tr></thead><tbody class='divide-y divide-white/[0.03]'>{}</tbody></table></div></div></div><div id='tab-rpg' class='tab-content'><div class='card-glass rounded-2xl overflow-hidden'><div class='px-6 py-5 border-b border-white/5'><div class='flex items-center gap-3'><div class='w-8 h-8 rounded-lg bg-cyan-500/20 flex items-center justify-center text-sm'>⚔️</div><div><div class='font-display font-bold text-sm tracking-wider text-gray-200'>RPG</div><div class='text-[10px] text-gray-600'>{} players</div></div></div></div><div class='overflow-x-auto'><table class='w-full text-sm'><thead class='text-gray-500 uppercase text-[10px] tracking-[0.15em] border-b border-white/5'><tr><th class='py-3 px-5 text-left'>Rank</th><th class='py-3 px-5 text-left'>Player</th><th class='py-3 px-5 text-left'><span class='text-cyan-400'>Level</span></th><th class='py-3 px-5 text-left'>EXP</th></tr></thead><tbody class='divide-y divide-white/[0.03]'>{}</tbody></table></div></div></div></main><footer class='relative text-center py-8 text-gray-700 text-xs border-t border-white/5 mt-8'><div class='flex items-center justify-center gap-4'><span>TAHBOТ ENGINE</span><span class='w-1 h-1 rounded-full bg-gray-700'></span><span>Groq AI</span><span class='w-1 h-1 rounded-full bg-gray-700'></span><a href='https://admin.themostpoordev.top' class='hover:text-purple-400 transition-colors'>Admin</a></div></footer><script>{}</script></body></html>",
        STYLE,
        total_users, total_msgs, total_econ, total_rpg,
        status_cards,
        top_rude, top_lewd, top_tim,
        total_users, rows,
        total_econ, econ_rows,
        total_rpg, rpg_rows,
        SCRIPT,
    );
    Html(html)
}
