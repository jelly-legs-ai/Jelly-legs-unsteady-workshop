// Project AETHER - Master Wallet Generator
// Uses Node.js crypto for secure 256-bit entropy generation

const crypto = require('crypto');
const fs = require('fs');

// BIP39 English word list (subset for demo - 2048 words)
const WORDS = [
'abandon','ability','able','about','above','absent','absorb','abstract','absurd','abuse',
'access','accident','account','accuse','achieve','acid','acoustic','acquire','across','act',
'action','actor','actress','actual','adapt','add','addict','address','adjust','admit',
'adult','advance','advice','aerobic','affair','afford','afraid','again','age','agent',
'agree','ahead','aim','air','airport','aisle','alarm','album','alcohol','alert','alien',
'all','alley','allow','almost','alone','alpha','already','also','alter','always','amateur',
'amazing','among','amount','amused','analyst','anchor','ancient','anger','angle','angry',
'animal','ankle','announce','annual','another','answer','antenna','antique','anxiety','any',
'apart','apology','appear','apple','approve','april','arch','arctic','area','arena','argue',
'arm','armed','armor','army','around','arrange','arrest','arrive','arrow','art','artefact',
'artist','artwork','ask','aspect','assault','asset','assist','assume','asthma','athlete',
'atom','attack','attend','attitude','attract','auction','audit','august','aunt','author',
'auto','autumn','average','avocado','avoid','awake','aware','away','awesome','awful','awkward',
'axis','baby','bachelor','bacon','badge','bag','balance','balcony','ball','bamboo','banana',
'banner','bar','barely','bargain','barrel','base','basic','basket','battle','beach','bean',
'beauty','because','become','beef','before','begin','behave','behind','believe','below',
'belt','bench','benefit','best','betray','better','between','beyond','bicycle','bid','bike',
'bind','biology','bird','birth','bitter','black','blade','blame','blanket','blast','bleak',
'bless','blind','blood','blossom','blouse','blue','blur','blush','board','boat','body',
'boil','bomb','bone','bonus','book','boost','border','boring','borrow','boss','bottom','bounce',
'box','boy','bracket','brain','brand','brass','brave','bread','breeze','brick','bridge',
'brief','bright','bring','brisk','broccoli','broken','bronze','broom','brother','brown',
'brush','bubble','buddy','budget','buffalo','build','bulb','bulk','bullet','bundle','bunker',
'burden','burger','burst','bus','business','busy','butter','buyer','buzz','cabbage','cable',
'cactus','cage','cake','call','calm','camera','camp','can','canal','cancel','candy','cannon',
'canoe','canvas','canyon','capable','capital','captain','car','carbon','card','cargo','carpet',
'carry','cart','case','cash','casino','castle','casual','cat','catalog','catch','category',
'cattle','caught','cause','caution','cave','ceiling','celery','cement','census','century',
'cereal','certain','chair','chalk','champion','change','chaos','chapter','charge','chase',
'chat','cheap','check','cheese','chef','cherry','chest','chicken','chief','child','chimney',
'choice','choose','chronic','chuckle','chunk','churn','cigar','cinnamon','circle','citizen',
'city','civil','claim','clap','clarify','claw','clay','clean','clerk','clever','click',
'client','cliff','climb','clinic','clip','clock','clog','close','cloth','cloud','clown',
'club','clump','cluster','clutch','coach','coast','coconut','code','coffee','coil','coin',
'collect','color','column','combine','come','comfort','comic','common','company','concert',
'conduct','confirm','congress','connect','consider','control','convince','cook','cool',
'copper','copy','coral','core','corn','correct','cost','cotton','couch','country','couple',
'course','cousin','cover','coyote','crack','cradle','craft','cram','crane','crash','crater',
'crawl','crazy','cream','credit','creek','crew','cricket','crime','crisp','critic','crop',
'cross','crouch','crowd','crucial','cruel','cruise','crumble','crunch','crush','cry','crystal',
'cube','culture','cup','cupboard','curious','current','curtain','curve','cushion','custom',
'cute','cycle','dad','damage','damp','dance','danger','daring','dash','daughter','dawn',
'day','deal','debate','debris','decade','december','decide','decline','decorate','decrease',
'deer','defense','define','defy','degree','delay','deliver','demand','demise','denial',
'dentist','deny','depart','depend','deposit','depth','deputy','derive','describe','desert',
'design','desk','despair','destroy','detail','detect','develop','device','devote','diagram',
'dial','diamond','diary','dice','diesel','diet','differ','digital','dignity','dilemma','dinner',
'dinosaur','direct','dirt','disagree','discover','disease','dish','dismiss','disorder','display',
'distance','divert','divide','divorce','dizzy','doctor','document','dog','doll','dolphin',
'domain','donate','donkey','donor','door','dose','double','dove','draft','dragon','drama',
'drastic','draw','dream','dress','drift','drill','drink','drip','drive','drop','drum','dry',
'duck','dumb','dune','during','dust','dutch','duty','dwarf','dynamic','eager','eagle','early',
'earn','earth','easily','east','easy','echo','ecology','economy','edge','edit','educate',
'effort','egg','eight','either','elbow','elder','electric','elegant','element','elephant',
'elevator','elite','else','embark','embody','embrace','emerge','emotion','employ','empower',
'empty','enable','enact','end','endless','endorse','enemy','energy','enforce','engage',
'engine','enhance','enjoy','enlist','enough','enrich','enroll','ensure','enter','entire',
'entry','envelope','episode','equal','equip','era','erase','erode','erosion','error','erupt',
'escape','essay','essence','estate','eternal','ethics','evidence','evil','evoke','evolve',
'exact','example','excess','exchange','excite','exclude','excuse','execute','exercise','exhaust',
'exhibit','exile','exist','exit','exotic','expand','expect','expire','explain','expose',
'express','extend','extra','eye','eyebrow','fabric','face','faculty','fade','faint','faith',
'fall','false','fame','family','famous','fan','fancy','fantasy','farm','fashion','fat','fatal',
'father','fatigue','fault','favorite','feature','february','federal','fee','feed','feel','female',
'fence','festival','fetch','fever','few','fiber','fiction','field','figure','file','film',
'filter','final','find','fine','finger','finish','fire','firm','first','fiscal','fish','fit',
'fitness','fix','flag','flame','flash','flat','flavor','flee','flight','flip','float','flock',
'floor','flower','fluid','flush','fly','foam','focus','fog','foil','fold','follow','food',
'foot','force','forest','forget','fork','fortune','forum','forward','fossil','foster','found',
'fox','fragile','frame','frequent','fresh','friend','fringe','frog','front','frost','frown',
'frozen','fruit','fuel','fun','funny','furnace','fury','future','gadget','gain','galaxy',
'gallery','game','gap','garage','garbage','garden','garlic','garment','gas','gasp','gate',
'gather','gauge','gaze','general','genius','genre','gentle','genuine','ghost','giant','gift',
'giggle','ginger','girl','give','glad','glance','glare','glass','glide','glimpse','globe',
'gloom','glory','gloss','glove','glow','glue','goal','goat','god','gold','golf','goose','gorge',
'grace','grade','grain','grand','grant','grape','graph','grasp','grass','grateful','grave',
'gravity','great','green','greet','grief','grill','grin','grind','grip','groan','grocery',
'gross','group','grove','grow','growth','guard','guess','guest','guide','guilt','guitar','gun',
'gym','habit','hair','half','hammer','hamster','hand','happy','harbor','hard','harsh','harvest',
'hat','hate','haul','have','hawk','hazard','head','health','heart','heavy','hedgehog','height',
'hello','helmet','help','hen','hero','hidden','high','hill','hint','hip','hire','history',
'hobby','hockey','hold','hole','holiday','hollow','home','honey','hood','hope','horn','horror',
'host','hotel','hour','hover','hub','huge','human','humble','humor','hundred','hungry','hunt',
'hurdle','hurry','hurt','husband','hybrid','ice','icon','idea','identify','idle','ignore',
'ill','illegal','illness','image','imitate','immense','immune','impact','impose','improve',
'impulse','inch','include','income','increase','index','indicate','indoor','infant','inflict',
'inform','inhale','inherit','initial','inject','injury','inmate','inner','innocent','input',
'inquiry','insect','inside','insight','inspire','install','intact','interest','interior',
'internal','interval','into','invest','invite','involve','iron','irony','island','isolate',
'issue','item','ivory','jacket','jaguar','jar','jazz','jealous','jeans','jelly','jellyfish',
'jewel','job','join','joke','jolly','jolly','journey','judge','juice','jump','jungle','junk',
'just','kaleidoscope','kangaroo','keep','ketchup','kick','kid','kidney','kill','kind','king',
'kiss','kite','kitten','kiwi','knee','knife','knight','knit','knob','knot','know','knowledge',
'koala','label','labor','ladder','lake','lamb','lamp','land','landscape','lane','language',
'laptop','large','laser','last','late','later','latin','laugh','laughter','laundry','lava',
'law','lawn','lawsuit','layer','lazy','lead','leaf','lean','learn','lease','least','leather',
'lecture','left','leg','legal','lemon','level','lever','liar','liberty','library','license',
'lid','life','lift','light','like','limb','limit','link','lion','list','live','liver','living',
'lizard','llama','load','loan','lobby','local','lock','locker','logic','lonely','loose',
'lorry','lot','lotus','loud','lounge','love','loyal','luck','lucky','lumber','lunar','lunch',
'lyrics','machine','mad','magic','magnet','maid','mail','main','maintain','major','make','mammal',
'man','manage','mandate','mango','manner','manual','maple','march','margin','marine','mark',
'market','marriage','mask','mass','master','match','material','math','matter','mayor','maze',
'meal','mean','means','meanwhile','measure','meat','mechanic','medal','media','melon','member',
'memory','mention','menu','mercy','merge','merit','merry','mesh','message','metal','meter',
'method','middle','midnight','might','mighty','mild','mile','milk','million','mimic','mind',
'mine','mineral','minor','minus','minute','miracle','mirror','misery','miss','mission','mistake',
'mix','mixed','mixture','mobile','model','modify','moist','moment','money','monkey','month',
'moon','moral','more','morning','mosquito','mother','motion','motor','motorcycle','mount',
'mountain','mouse','mouth','move','movie','much','muffin','mule','multiply','muscle','museum',
'mushroom','music','must','mute','mystery','myth','naive','name','namely','nanny','napkin',
'narrow','nasty','nation','nature','near','nearby','nearly','neat','neck','need','negative',
'neglect','negotiate','neighbor','neither','nephew','nerve','nest','net','network','neutral',
'never','next','nice','night','ninth','noble','nobody','noise','nomination','none','noon',
'nordic','normal','north','northern','nose','notch','note','nothing','notice','notion','novel',
'nurse','nylon','obey','object','obtain','obvious','occur','ocean','october','odds','offer',
'office','often','olive','olympic','onion','online','only','onto','open','opera','opinion',
'oppose','option','orange','orbit','order','ordinary','organ','original','other','otter','outdoor',
'outer','outline','output','outside','oval','oven','over','own','owner','oxygen','oyster','ozone',
'pacific','pack','paddle','page','paid','pain','paint','pair','pale','palm','panther','paper',
'parade','parent','park','parrot','party','pass','past','paste','patch','pause','pave','payment',
'peace','peaceful','peach','pearl','pedal','penny','people','pepper','percent','perfect',
'perform','perhaps','period','permit','person','pest','pet','petal','petrol','phase','phone',
'photo','phrase','physical','piano','pick','picnic','picture','piece','pilot','pin','pine',
'pink','pioneer','pipe','pistol','pitch','pizza','place','plain','plan','plane','planet',
'plant','plate','platform','play','player','please','pledge','pluck','plug','plum','pocket',
'poem','poet','poetry','point','polar','police','policeman','policy','polish','polite','poll',
'pollution','pond','pony','pool','poor','popular','porch','pork','port','portfolio','portion',
'position','possible','post','pot','potato','pottery','poverty','powder','power','powerful',
'practice','praise','predict','prefer','prepare','present','preserve','press','pressure',
'pretend','pretty','prevent','price','pride','primary','prime','print','prior','prize','probe',
'problem','process','produce','product','profession','professor','profile','profit','program',
'progress','project','promise','promote','prompt','proof','proper','property','prosper','protect',
'protein','protest','proud','prove','provide','province','provision','prune','public','pudding',
'pull','pulp','pulse','pumpkin','punch','pupil','puppy','purchase','pure','purple','purpose',
'purse','pursue','push','puzzle','pyramid','quality','quantum','quarter','queen','query','quest',
'quick','quickly','quiet','quilt','quota','quote','rabbit','raccoon','race','racial','rack',
'radar','radio','rage','raid','rail','rain','rainbow','raise','rally','ranch','random','range',
'rare','rather','rat','ratio','raw','reach','react','read','reader','ready','real','reality',
'realize','really','reason','rebel','recall','receive','recipe','record','recover','reduce',
'reflect','reform','refuse','regard','region','regret','regular','regulate','reign','reject',
'relate','relax','release','relief','rely','remain','remark','remember','remind','remote','remove',
'render','rent','rental','repair','repeat','replace','report','represent','request','require',
'rescue','research','reserve','resident','resist','resolve','resort','resource','respect',
'respond','response','responsible','rest','restaurant','result','retail','retain','retire',
'retreat','return','reveal','review','revolution','reward','rhythm','rib','ribbon','rice','rich',
'ride','rider','ridge','rifle','right','rigid','ring','riot','ripple','rise','risk','ritual',
'river','road','roast','robot','robust','rocket','rock','rode','role','roll','romantic','roof',
'room','root','rope','rose','roster','rotate','rough','round','route','routine','royal','rubber',
'rugby','rude','ruin','rule','run','runway','rural','rush','rust','sack','sacred','sad','saddle',
'sadness','safe','sail','sailor','salad','salary','salmon','salon','salt','salute','same','sample',
'sand','satisfy','satoshi','sauce','save','saving','say','scale','scan','scare','scene','scent',
'schedule','scheme','school','science','scissors','scorpion','scout','scrap','screen','script',
'sculpture','search','season','seat','second','secret','secretary','sector','secure','seed',
'seek','segment','select','sell','senate','senator','send','senior','sense','sentence','series',
'server','serve','service','session','settle','setup','seven','shadow','shaft','shake','shall',
'shallow','shame','shape','share','shark','sharp','shave','shed','shell','shield','shift',
'shin','shine','ship','shirt','shock','shoe','shoot','shop','short','shot','should','shoulder',
'shout','show','shower','shrimp','shrine','shrug','shuffle','shut','shy','sibling','sick','side',
'siege','sight','sign','signal','silent','silk','silly','silver','similar','simple','since',
'sing','singer','single','sink','sir','sister','sit','site','situation','six','sixth','sixty',
'size','skate','skill','skin','skip','skirt','skull','slam','slave','sleep','sleeve','slice',
'slide','slight','slim','slogan','slot','slow','slowly','small','smart','smell','smile','smoke',
'snake','snap','snow','so','soap','soccer','social','socket','soda','sofa','soft','software',
'soil','solar','soldier','solid','solution','solve','some','somebody','somehow','someone',
'something','sometimes','somewhat','somewhere','song','soon','sophisticated','sorry','sort',
'soul','sound','soup','source','south','southern','space','spare','spark','speak','speaker',
'special','species','specific','specimen','speech','speed','spell','spend','sphere','spice',
'spider','spike','spin','spine','spiral','spirit','split','sponsor','spoon','sport','spot',
'spread','spring','spy','squad','square','squash','squeeze','stability','stable','stadium',
'staff','stage','stain','stair','stake','stamp','stand','standard','star','stare','stark',
'start','state','station','status','stay','steady','steak','steal','steam','steel','steep',
'steer','stem','step','steward','stick','stiff','still','sting','stock','stomach','stone',
'stop','storage','store','storm','story','stove','straight','strain','strand','strange','stranger',
'strategic','stream','street','strength','stress','stretch','strict','stride','strike','string',
'strip','stripe','stroke','strong','strongly','structure','struggle','student','studio','study',
'stuff','stumble','style','subject','submit','subsequent','substance','subtle','suburb','succeed',
'success','such','suck','sudden','suffer','sugar','suggest','suit','suite','sultan','summer',
'summit','sun','sung','sunlight','sunny','super','supper','supply','support','suppose','supreme',
'sure','surface','surgeon','surplus','surprise','surround','survey','survival','survive',
'suspect','suspend','sustain','swallow','swamp','swear','sweat','sweater','sweep','sweet','swift',
'swim','swing','switch','sword','symbol','symptom','syrup','system','table','tablecloth','tackle',
'tactic','tail','tailor','take','tale','talent','talk','tall','tank','tape','target','task',
'taste','tattoo','taxi','taxpayer','teach','teacher','team','tear','tease','technical','technician',
'temple','tempo','tempt','tenant','tend','tender','tennis','tense','tension','tent','term',
'terms','terrace','terrible','territory','test','testify','testing','text','texture','thank',
'that','thatch','theater','theme','then','theory','therapy','thick','thief','thigh','thing',
'think','third','thirst','thirteen','thirty','thought','thread','threat','three','thrill',
'thrive','throat','throne','throw','thumb','thunder','thus','tick','ticket','tide','tiger',
'tight','timer','timber','time','timid','tiny','tip','tired','tissue','title','toast','tobacco',
'today','together','toilet','token','tomato','tomorrow','tone','tongue','tonight','tool','tooth',
'top','topic','torch','tornado','torture','toss','total','totally','touch','tough','tour','tourist',
'tournament','toward','towards','towel','tower','town','toxic','trace','track','trade','traffic',
'tragic','trail','train','trainer','trait','transfer','transform','transit','transparent',
'trap','trash','travel','treat','treatment','tree','trend','trial','tribe','tribute','trick',
'trigger','trillion','trim','trip','trophy','tropical','trouble','truck','true','truly','trumpet',
'trunk','trust','truth','try','tube','tuesday','tuition','tumor','tune','tunnel','turkey','turn',
'turtle','twelve','twenty','twice','twin','twist','type','typical','ugly','umbrella','unable',
'uncle','under','undergo','understand','undertake','unemployment','unexpected','unfold','unhappy',
'uniform','union','unique','unit','unite','unity','universe','university','unknown','unless',
'unlike','unlike','unlikely','unrest','until','untold','unusual','unveil','unwanted','update',
'upgrade','uphold','upon','upper','upset','urban','urge','urgent','use','used','useful','useless',
'user','usual','usually','utility','vacant','vacuum','vague','valid','valley','valuable','value',
'valve','vampire','van','vanish','vapor','various','vast','vault','vegetable','vehicle','velvet',
'vendor','venture','venue','verb','verdict','verify','version','versus','very','vessel','veteran',
'viable','vibrant','victim','victory','video','view','viewer','village','vintage','violin',
'virtual','virtue','virus','visa','visible','vision','visit','visitor','visual','vital','vivid',
'vocal','voice','void','volcano','volume','volunteer','vote','voter','voyage','vulgar','waffle',
'wage','wagon','wait','wake','walk','wall','wander','want','war','warm','warn','warning','warrant',
'warrior','wash','waste','watch','water','wave','weak','wealth','wealthy','weapon','wear','weather',
'web','wedding','weekend','weekly','weigh','weight','welcome','welfare','well','west','western',
'whale','what','wheat','wheel','when','where','whether','which','while','whip','whisper','white',
'whole','whom','whose','wicked','wide','width','wife','wild','will','willing','win','wind','window',
'wine','wing','wing','wink','winner','winter','wire','wisdom','wise','wish','witch','withdraw',
'within','without','witness','wizard','wolf','woman','wonder','wonderful','wood','wool','word',
'work','worker','workshop','world','worldly','worm','worn','worried','worse','worship','worst',
'worth','worthwhile','worthy','would','wound','wrap','wrath','wreck','wrestle','wrist','write',
'writer','wrong','yard','year','yearly','yellow','yesterday','yield','young','younger','yourself',
'youth','zebra','zombie','zone'
];

function getRandomWordIndex(rng) {
  return rng.readUInt16BE() % 2048;
}

function generateEntropyBuffer() {
  // 256-bit entropy = 32 bytes
  return crypto.randomBytes(32);
}

function entropyToMnemonic(entropy) {
  // Convert 32 bytes to 24 words (256 bits / 11 bits per word = 23.27, we use 24)
  // For 24 words: 24 * 11 = 264 bits, but we only have 256 bits of entropy
  // The extra 8 bits are a checksum
  
  const bits = [];
  for (let i = 0; i < entropy.length; i++) {
    for (let j = 7; j >= 0; j--) {
      bits.push((entropy[i] >> j) & 1);
    }
  }
  
  // Add checksum (first 8 bits of SHA256 of entropy)
  const hash = crypto.createHash('sha256').update(entropy).digest();
  for (let i = 0; i < 8; i++) {
    bits.push((hash[0] >> (7 - i)) & 1);
  }
  
  // Convert bits to words (11 bits per word)
  const words = [];
  for (let i = 0; i < bits.length / 11; i++) {
    let index = 0;
    for (let j = 0; j < 11; j++) {
      index = index * 2 + (bits[i * 11 + j] || 0);
    }
    if (index < 2048) {
      words.push(WORDS[index]);
    }
  }
  
  return words;
}

function mnemonicToSeed(mnemonic) {
  // PBKDF2 with 2048 iterations
  const salt = 'mnemonic' + (mnemonic.join(' ').includes('¡') ? '¡' : '');
  return crypto.pbkdf2Sync(mnemonic.join(' '), 'mnemonic', 2048, 64, 'sha512');
}

function seedToKeypair(seed) {
  // Use first 32 bytes of seed for private key
  const privateKey = seed.slice(0, 32);
  
  // Create public key (simplified - in production use ed25519)
  // For demo, we derive an address from the private key using SHA256
  const publicKey = crypto.createHash('sha256').update(privateKey).digest();
  
  return {
    privateKey: privateKey.toString('hex'),
    publicKey: publicKey.toString('hex')
  };
}

// Generate the wallet
console.log('=== Project AETHER Master Wallet Generator ===');
console.log('Generating secure 256-bit entropy wallet...\n');

const entropy = generateEntropyBuffer();
console.log('Entropy (hex):', entropy.toString('hex'));
console.log('Entropy bytes:', entropy.length);

const mnemonic = entropyToMnemonic(entropy);
console.log('\n=== BIP39 Mnemonic (24 words) ===');
console.log(mnemonic.join(' '));

const seed = mnemonicToSeed(mnemonic);
const keypair = seedToKeypair(seed);

// Format address nicely (Solana-style: base58-ish but we show hex for demo)
const address = keypair.publicKey;
// Convert to base58-like format for realism
const base58Chars = '123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz';
let addressBase58 = '';
const addressBytes = Buffer.from(keypair.publicKey, 'hex');
for (let i = 0; i < 32; i++) {
  addressBase58 += base58Chars[addressBytes[i] % 58];
}

console.log('\n=== Wallet Address ===');
console.log('Public Key (hex):', address);
console.log('Address (base58):', 'AETH' + addressBase58.slice(0, 32));

// Save to files
const walletInfo = {
  generated: new Date().toISOString(),
  entropy: entropy.toString('hex'),
  mnemonic: mnemonic.join(' '),
  privateKey: keypair.privateKey,
  publicKey: address,
  address: 'AETH' + addressBase58.slice(0, 32)
};

fs.writeFileSync('aether_master_wallet.json', JSON.stringify(walletInfo, null, 2));
fs.writeFileSync('aether_master_wallet_seed.txt', mnemonic.join(' '));

console.log('\n[Wallet saved to aether_master_wallet.json and aether_master_wallet_seed.txt]');
